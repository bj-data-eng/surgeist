use super::{
    Error, Golden,
    capture_authority::{PathDigest, SpecializedEvidence, sha256},
};

#[path = "../../bin/surgeist-layout-generate/measurement/horizontal_br_font_band_profile.rs"]
mod profile;
pub(crate) use profile::{HELPER_SHA256, REQUEST, SCRIPT, SCRIPT_PATH, SOURCE, SOURCE_SHA256};

pub(crate) fn evidence() -> SpecializedEvidence {
    SpecializedEvidence::HorizontalBrFontBand {
        proof_script: PathDigest {
            path: SCRIPT_PATH.to_owned(),
            sha256: sha256(SCRIPT.as_bytes()),
        },
    }
}

/// Execute the projection after the publisher or registry authenticates this
/// profile and verifies the retained observations.
pub(crate) fn compare_authenticated(golden: &Golden) -> Result<(), Error> {
    super::compare_captured_with_projection(
        golden,
        &mut super::NumericComparisonSink::Strict,
        super::BrFontBandProjection::HorizontalPinned,
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
            "horizontal BR capture changed the historical box, control or font-band observations",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn historical_break_band_must_retain_every_selected_coordinate() {
        let retained = Golden::parse(include_str!(
            "xml/block/block_br_inline_block_metrics__border_box_ltr.xml"
        ))
        .unwrap();
        verify_historical(&retained, &retained).unwrap();
        let error = compare_authenticated(&retained).unwrap_err();
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
