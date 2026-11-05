//! Canonical selected resources and normalized instance construction.

use super::*;

/// A matched resource and its selected synthesis/instance suggestions. Authored
/// variations have not yet been applied; this is not a canonical coordinate set.
pub(super) struct ResourceSelection {
    pub(super) font: FontData,
    pub(super) synthesis: FontSynthesis,
    pub(super) suggested_variations: Vec<parlance::FontVariation>,
}

/// Actual retained resource, used size, synthesis and completed normalized
/// coordinates. The same owned instance moves into its glyph-free receipt.
pub(crate) struct CanonicalFontInstance {
    pub(crate) font: FontData,
    pub(crate) font_size: f32,
    pub(crate) synthesis: FontSynthesis,
    pub(crate) coords: Vec<i16>,
}

pub(super) fn selected_resource(
    selected: SelectedResource,
    size: f32,
) -> Result<ResourceSelection> {
    match selected {
        SelectedResource::Matched(selected) => {
            let (font, synthesis) = matched_resource(&selected, size)?;
            let suggested_variations = selected.synthesis().variation_settings().collect();
            Ok(ResourceSelection {
                font,
                synthesis,
                suggested_variations,
            })
        }
        SelectedResource::Control(font) => Ok(ResourceSelection {
            font,
            synthesis: FontSynthesis::default(),
            suggested_variations: Vec::new(),
        }),
    }
}

fn matched_resource(selected: &FontCandidate, size: f32) -> Result<(FontData, FontSynthesis)> {
    let synthesis = FontSynthesis::resolve(
        size,
        selected.synthesis().skew(),
        selected.synthesis().embolden(),
    )?;
    Ok((
        FontData::from_face(selected.face().clone(), selected.publication().clone()),
        synthesis,
    ))
}

fn canonical_variations<'a>(
    suggested: impl Iterator<Item = parlance::FontVariation> + 'a,
    authored: &'a [parlance::FontVariation],
) -> impl Iterator<Item = surgeist_shaper::Variation> + 'a {
    suggested
        .chain(authored.iter().copied())
        .map(|value| surgeist_shaper::Variation {
            tag: surgeist_shaper::Tag::new(&value.tag.to_bytes()),
            value: value.value,
        })
}

pub(super) fn canonical_instance(
    font_ref: &surgeist_shaper::FontRef<'_>,
    key: &cache::InstanceKey,
    retained_coords: Option<&[i16]>,
) -> surgeist_shaper::ShaperInstance {
    if let Some(coords) = retained_coords {
        surgeist_shaper::ShaperInstance::from_coords(
            font_ref,
            coords
                .iter()
                .copied()
                .map(surgeist_shaper::NormalizedCoord::from_bits),
        )
    } else {
        surgeist_shaper::ShaperInstance::from_variations(
            font_ref,
            canonical_variations(key.suggested_variations.iter().copied(), &key.variations),
        )
    }
}

/// Matches a scope font through its explicit family request. Resource and
/// variation completion shares the body path without constructing a glyph run.
pub(crate) fn scope_font(
    mut query: FontQuery<'_>,
    style: &crate::engine::resolve::ResolvedFontStyle,
    budget: &mut crate::processing::contextual::alignment::ScopeAlignmentBudget,
    pinned: &[(usize, usize)],
) -> Result<CanonicalFontInstance> {
    budget.font_query()?;
    let attrs = FontAttributes::new(style.font_width, style.font_style, style.font_weight);
    match query.visit_candidates(&style.font_family, attrs, |font| {
        ControlFlow::Break(complete_match(
            font,
            style.font_size,
            &style.font_variations,
            budget,
            pinned,
        ))
    })? {
        ControlFlow::Break(result) => result,
        ControlFlow::Continue(()) => Err(no_font()),
    }
}

fn work_overflow() -> Error {
    super::super::scope_alignment::refusal(crate::ScopeAlignmentError::Overflow {
        kind: crate::ScopeAlignmentLimitKind::StructuralWork,
    })
}

/// The coordinate-only part of ShaperInstance::set_variations: the same fvar/
/// avar operation, suggestion-before-authored ordering and empty default vector.
/// The owned output and normalization scratch are fallible feature buffers;
/// no InstanceKey/settings clone, sample glyph, plan or glyph invocation is
/// constructed. Opaque font-library internals are not a Text payload/RSS census.
fn scope_coords(
    font: &surgeist_shaper::FontRef<'_>,
    suggested: impl ExactSizeIterator<Item = parlance::FontVariation>,
    authored: &[parlance::FontVariation],
    budget: &mut crate::processing::contextual::alignment::ScopeAlignmentBudget,
) -> Result<Vec<i16>> {
    use skrifa::raw::types::Fixed;
    budget.work(2)?;
    for tag in [*b"fvar", *b"avar"] {
        if let Some(data) = font.table_data(surgeist_shaper::Tag::new(&tag)) {
            budget.work(data.as_bytes().len())?;
        }
    }
    let mut result = Vec::new();
    let Ok(fvar) = font.fvar() else {
        return Ok(result);
    };
    let count = usize::from(fvar.axis_count());
    let settings = suggested
        .len()
        .checked_add(authored.len())
        .ok_or_else(work_overflow)?;
    budget.work(
        count
            .checked_mul(settings.checked_add(3).ok_or_else(work_overflow)?)
            .ok_or_else(work_overflow)?,
    )?;
    let mut normalized = Vec::new();
    budget.add_records(count)?;
    budget.reserve(&mut normalized, count)?;
    normalized.resize(count, surgeist_shaper::NormalizedCoord::ZERO);
    fvar.user_to_normalized(
        font.avar().ok().as_ref(),
        canonical_variations(suggested, authored)
            .map(|value| (value.tag, Fixed::from_f64(f64::from(value.value)))),
        &mut normalized,
    );
    if normalized
        .iter()
        .any(|value| *value != surgeist_shaper::NormalizedCoord::ZERO)
    {
        budget.add_records(count)?;
        budget.reserve(&mut result, count)?;
        result.extend(normalized.iter().map(|value| value.to_bits()));
    }
    let bytes = super::super::scope_alignment::accounting::bytes(&normalized)?;
    drop(normalized);
    budget.release_bytes(bytes)?;
    Ok(result)
}

/// Coverage-specific hypothetical matching shares canonical instance completion.
pub(super) fn character_font(
    mut query: FontQuery<'_>,
    style: &ResolvedStyle,
    ch: char,
    budget: &mut crate::processing::contextual::alignment::ScopeAlignmentBudget,
    pinned: &[(usize, usize)],
) -> Result<Option<CanonicalFontInstance>> {
    budget.font_query()?;
    budget.work(
        style
            .font_family
            .len()
            .checked_add(1)
            .ok_or_else(work_overflow)?,
    )?;
    let selected = query.visit_candidates(&style.font_family, attributes(style), |font| {
        let result = (|| -> Result<Option<CanonicalFontInstance>> {
            budget.work(1)?;
            if !font
                .charmap()
                .and_then(|map| map.glyph_id(ch))
                .is_some_and(|id| id != 0)
            {
                return Ok(None);
            }
            complete_match(
                font,
                style.font_size,
                &style.font_variations,
                budget,
                pinned,
            )
            .map(Some)
        })();
        match result {
            Ok(None) => ControlFlow::Continue(()),
            other => ControlFlow::Break(other),
        }
    })?;
    match selected {
        ControlFlow::Break(result) => result,
        ControlFlow::Continue(()) => Ok(None),
    }
}

fn complete_match(
    font: &FontCandidate,
    size: f32,
    variations: &[parlance::FontVariation],
    budget: &mut crate::processing::contextual::alignment::ScopeAlignmentBudget,
    pinned: &[(usize, usize)],
) -> Result<CanonicalFontInstance> {
    let (resource, synthesis) = matched_resource(font, size)?;
    let identity = (resource.bytes().as_ptr() as usize, resource.bytes().len());
    // Conservative duplicate-check admission, not a comparison trace.
    budget.work(pinned.len().checked_add(1).ok_or_else(work_overflow)?)?;
    if !pinned.contains(&identity) {
        budget.retain_bytes(identity.1)?;
    }
    let font_ref = surgeist_shaper::FontRef::from_index(resource.bytes(), resource.index())
        .map_err(|_| Error::new(ErrorCode::FontLoadFailed, "invalid selected font face"))?;
    let coords = scope_coords(
        &font_ref,
        font.synthesis().variation_settings(),
        variations,
        budget,
    )?;
    Ok(CanonicalFontInstance {
        font: resource,
        font_size: size,
        synthesis,
        coords,
    })
}
