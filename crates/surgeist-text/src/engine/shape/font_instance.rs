//! Canonical selected resources and normalized instance construction.

use super::*;

/// A matched resource and its selected synthesis/instance suggestions. Authored
/// variations have not yet been applied; this is not a canonical coordinate set.
pub(super) struct ResourceSelection {
    pub(super) font: FontData,
    pub(super) synthesis: FontSynthesis,
    pub(super) suggested_variations: Vec<(surgeist_shaper::Tag, f32)>,
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
    catalogue: &crate::FontCatalogueIdentity,
) -> Result<ResourceSelection> {
    match selected {
        SelectedResource::Matched(selected) => {
            let (font, synthesis) = matched_resource(&selected, size, catalogue)?;
            let suggested_variations = selected.synthesis.variation_settings().to_vec();
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

fn matched_resource(
    selected: &QueryFont,
    size: f32,
    catalogue: &crate::FontCatalogueIdentity,
) -> Result<(FontData, FontSynthesis)> {
    let synthesis = FontSynthesis::resolve(
        size,
        selected.synthesis.skew(),
        selected.synthesis.embolden(),
    )?;
    Ok((
        FontData::from_blob(selected.blob.clone(), selected.index, catalogue.clone()),
        synthesis,
    ))
}

fn canonical_variations<'a>(
    suggested: &'a [(surgeist_shaper::Tag, f32)],
    authored: &'a [parlance::FontVariation],
) -> impl Iterator<Item = surgeist_shaper::Variation> + 'a {
    suggested
        .iter()
        .map(|(tag, value)| surgeist_shaper::Variation {
            tag: *tag,
            value: *value,
        })
        .chain(authored.iter().map(|value| surgeist_shaper::Variation {
            tag: surgeist_shaper::Tag::new(&value.tag.to_bytes()),
            value: value.value,
        }))
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
            canonical_variations(&key.suggested_variations, &key.variations),
        )
    }
}

/// Actual scope-font matching uses its captured script and no character coverage.
/// It shares resource/variation construction with real body runs without making
/// an empty run or invoking the glyph provider.
pub(crate) fn scope_font(
    mut query: Query<'_>,
    style: &crate::engine::resolve::ResolvedFontStyle,
    script: Script,
    catalogue: &crate::FontCatalogueIdentity,
    budget: &mut crate::processing::contextual::alignment::ScopeAlignmentBudget,
    pinned: &[(usize, usize)],
) -> Result<CanonicalFontInstance> {
    budget.font_query()?;
    // Same canonical family/attribute/fallback inputs as an ordinary non-emoji
    // body query, without a full paint style or synthetic coverage characters.
    query.set_families(style.font_family.iter().copied().map(QueryFamily::Id));
    query.set_attributes(Attributes {
        width: style.font_width,
        weight: style.font_weight,
        style: style.font_style,
    });
    query.set_fallbacks(fontique::FallbackKey::new(
        convert::script_to_fontique(script, &AnalysisDataSources),
        style.locale.as_ref(),
    ));
    let mut selected = None;
    query.matches_with(|font| {
        selected = Some((|| {
            let (resource, synthesis) = matched_resource(font, style.font_size, catalogue)?;
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
                font.synthesis.variation_settings(),
                &style.font_variations,
                budget,
            )?;
            Ok(CanonicalFontInstance {
                font: resource,
                font_size: style.font_size,
                synthesis,
                coords,
            })
        })());
        QueryStatus::Stop
    });
    selected.ok_or_else(no_font)?
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
    suggested: &[(surgeist_shaper::Tag, f32)],
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
