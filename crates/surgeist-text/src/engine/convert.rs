// Copyright 2024 the Parley Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use super::analysis::AnalysisDataSources;

use icu_properties::props::Script;

pub(crate) fn script_to_fontique(
    script: Script,
    analysis_data_sources: &AnalysisDataSources,
) -> fontique::Script {
    analysis_data_sources
        .script_short_name()
        .get(script)
        .unwrap_or("Zzzz")
        .parse()
        .unwrap_or(fontique::Script::UNKNOWN)
}

pub(crate) fn script_to_shaper(script: fontique::Script) -> surgeist_shaper::Script {
    surgeist_shaper::Script::from_iso15924_tag(surgeist_shaper::Tag::new(&script.to_bytes()))
        .unwrap_or(surgeist_shaper::script::UNKNOWN)
}
