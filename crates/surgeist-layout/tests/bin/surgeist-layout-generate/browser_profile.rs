//! Shared Layout-owned browser declaration and fixed lifecycle policy.

use serde::Deserialize;
use surgeist_generator::RelativePath;
use surgeist_generator::browser::{BrowserLaunch, BrowserSettings};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BrowserManifest {
    source: String,
    version: String,
    version_output: String,
    cache_root: String,
    provenance_format: String,
    launch: Launch,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Launch {
    batch_size: usize,
    navigation_timeout_ms: u64,
    dom_poll_interval_ms: u64,
    retry_count: usize,
    job_order: String,
    retry_error_class: String,
    profile_scope: String,
    page_scope: String,
    disable_default_args: bool,
    disable_cache: bool,
    arguments: Vec<String>,
}

impl BrowserManifest {
    pub(super) fn validate(&self) -> Result<(), String> {
        let launch = &self.launch;
        if launch.retry_count != 1
            || launch.job_order != "sorted-sequential"
            || launch.retry_error_class != "open-load-reset-timeout"
            || launch.profile_scope != "per-batch-and-retry"
            || launch.page_scope != "per-job"
            || !launch.disable_default_args
            || !launch.disable_cache
        {
            return Err(
                "manifest browser lifecycle does not match the retained layout fixture protocol"
                    .to_string(),
            );
        }
        Ok(())
    }

    pub(super) fn settings(&self) -> Result<BrowserSettings, String> {
        let launch = &self.launch;
        let launch = BrowserLaunch::new(
            launch.batch_size,
            launch.navigation_timeout_ms,
            launch.dom_poll_interval_ms,
            launch.arguments.clone(),
        )
        .map_err(message)?;
        BrowserSettings::new(
            self.source.clone(),
            self.version.clone(),
            self.version_output.clone(),
            RelativePath::new(&self.cache_root).map_err(message)?,
            self.provenance_format.clone(),
            launch,
        )
        .map_err(message)
    }
}

fn message(error: impl std::fmt::Display) -> String {
    error.to_string()
}
