// Copyright 2024 the Parley Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use super::analysis::AnalysisDataSources;

use icu_properties::props::Script;

pub(crate) fn script_to_shaper(
    script: Script,
    analysis_data_sources: &AnalysisDataSources,
) -> surgeist_shaper::Script {
    analysis_data_sources
        .script_short_name()
        .get(script)
        .and_then(|name| <[u8; 4]>::try_from(name.as_bytes()).ok())
        .and_then(|tag| surgeist_shaper::Script::from_iso15924_tag(surgeist_shaper::Tag::new(&tag)))
        .unwrap_or(surgeist_shaper::script::UNKNOWN)
}
