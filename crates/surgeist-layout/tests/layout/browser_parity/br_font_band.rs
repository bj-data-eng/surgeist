use super::{
    BrFontBandProfile, Error, Golden,
    capture_authority::{PathDigest, SpecializedEvidence, sha256},
};

pub(crate) use crate::horizontal_br_font_band_profile::{
    REQUEST, SCRIPT, SCRIPT_PATH, SOURCE, SOURCE_SHA256,
};

pub(crate) fn horizontal_evidence() -> SpecializedEvidence {
    SpecializedEvidence::HorizontalBrFontBand {
        proof_script: PathDigest {
            path: SCRIPT_PATH.to_owned(),
            sha256: sha256(SCRIPT.as_bytes()),
        },
    }
}

pub(crate) fn vertical_evidence() -> SpecializedEvidence {
    use crate::vertical_br_font_band_profile as profile;
    SpecializedEvidence::VerticalBrFontBand {
        proof_script: PathDigest {
            path: profile::SCRIPT_PATH.to_owned(),
            sha256: sha256(profile::SCRIPT.as_bytes()),
        },
    }
}

pub(crate) fn empty_line_evidence() -> SpecializedEvidence {
    use crate::empty_line_br_font_band_profile as profile;
    SpecializedEvidence::EmptyLineBrFontBand {
        proof_script: PathDigest {
            path: profile::SCRIPT_PATH.to_owned(),
            sha256: sha256(profile::SCRIPT.as_bytes()),
        },
    }
}

/// Execute the projection after the publisher or registry authenticates this
/// profile and verifies the retained observations.
pub(crate) fn compare_authenticated(
    golden: &Golden,
    profile: BrFontBandProfile,
) -> Result<(), Error> {
    super::compare_captured_with_projection(
        golden,
        &mut super::NumericComparisonSink::Strict,
        Some(profile),
        |_, _| Ok(()),
    )
    .map(|_| ())
}

pub(crate) fn verify_historical(current: &Golden, historical: &Golden) -> Result<(), Error> {
    if current.name != historical.name
        || !current.use_rounding
        || !historical.use_rounding
        || current.expectations != historical.expectations
    {
        return Err(Error::new(
            "BR capture changed the historical box, control or font-band observations",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_line_bands_require_the_original_font_and_control_order() {
        let raw = include_str!(
            "captured/empty-line-br-font-band-qualification/xml/block/block_br_empty_lines_metrics__border_box_ltr.xml"
        );
        let golden = Golden::parse(raw).unwrap();
        compare_authenticated(&golden, BrFontBandProfile::EmptyLine).unwrap();
        let foreign = compare_authenticated(&golden, BrFontBandProfile::Horizontal).unwrap_err();
        assert!(
            foreign
                .to_string()
                .contains("no qualified pinned-font projection")
        );

        let document = roxmltree::Document::parse(raw).unwrap();
        let capture = document
            .descendants()
            .find(|node| node.has_tag_name("prepared-paragraph"))
            .unwrap()
            .attribute("capture")
            .unwrap();
        let original: serde_json::Value = serde_json::from_str(capture).unwrap();
        let parse = |value: &serde_json::Value| {
            super::super::prepared_source::Capture::parse(&value.to_string(), 2)
        };

        let mut unloaded = original.clone();
        unloaded["font"]["loaded"] = false.into();
        assert!(
            parse(&unloaded)
                .unwrap_err()
                .to_string()
                .contains("loading proof mismatch")
        );

        let mut foreign = original.clone();
        foreign["font"]["sha256"] = super::super::pinned_font::SHA256.into();
        assert!(
            parse(&foreign)
                .unwrap_err()
                .to_string()
                .contains("pinned resource")
        );

        let mut reordered = original;
        reordered["order"].as_array_mut().unwrap().swap(0, 1);
        assert!(
            parse(&reordered)
                .unwrap_err()
                .to_string()
                .contains("actual child order")
        );
    }

    #[test]
    fn historical_break_band_must_retain_every_selected_coordinate() {
        let retained = Golden::parse(include_str!(
            "xml/block/block_br_inline_block_metrics__border_box_ltr.xml"
        ))
        .unwrap();
        verify_historical(&retained, &retained).unwrap();
        let error = compare_authenticated(&retained, BrFontBandProfile::Horizontal).unwrap_err();
        assert!(error.to_string().contains("MissingRootContextAuthority"));
        let error = compare_authenticated(&retained, BrFontBandProfile::Vertical).unwrap_err();
        assert!(error.to_string().contains("MissingRootContextAuthority"));
        let error = compare_authenticated(&retained, BrFontBandProfile::EmptyLine).unwrap_err();
        assert!(error.to_string().contains("MissingRootContextAuthority"));

        for field in 0..4 {
            let mut changed = retained.clone();
            let band = changed.expectations.children[1]
                .browser_br_font_band
                .as_mut()
                .unwrap();
            match field {
                0 => band.x += 1.0,
                1 => band.y += 1.0,
                2 => band.width += 1.0,
                3 => band.height += 1.0,
                _ => unreachable!(),
            }
            assert!(verify_historical(&changed, &retained).is_err());
        }
    }
}
