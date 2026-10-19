//! Closed selection for the single populated native feature/import lifecycle.
use super::{COMBINED_NODES, SOURCE_NODES, Stage, TARGET_NODES, run_sample};
use crate::store::scale_tests::memory::selection::{
    Identity, Sample, checkpoint_config, optional, required,
};
use crate::store::scale_tests::{require_release, scale_metrics};
use surgeist_test::checkpoint::{Config, Gate};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Selection {
    sample: Sample,
}
impl Selection {
    fn parse(population: &str, managed: &str, sample: &str) -> Result<Self, &'static str> {
        if population != "native-features" {
            return Err("SURGEIST_MEMORY_POPULATION must be native-features");
        }
        let managed: usize = managed
            .parse()
            .map_err(|_| "invalid SURGEIST_MEMORY_MANAGED")?;
        if managed != COMBINED_NODES {
            return Err("native-features requires exactly 13 combined node identities");
        }
        Ok(Self {
            sample: Sample::parse(sample)?,
        })
    }
    fn identity(self) -> Identity {
        Identity::Native {
            population: "native-features",
            managed: COMBINED_NODES,
            sample: self.sample,
        }
    }
}
fn checkpoints(value: &str) -> Result<Vec<Stage>, &'static str> {
    if value == "none" {
        return Ok(Vec::new());
    }
    if value == "all" {
        return Ok(Stage::ALL.to_vec());
    }
    let mut stages = Vec::new();
    let mut previous = None;
    for name in value.split(',') {
        let ordinal = Stage::ALL
            .iter()
            .position(|stage| stage.name() == name)
            .ok_or("invalid native feature checkpoint")?;
        if previous.is_some_and(|previous| ordinal <= previous) {
            return Err("feature checkpoints must be distinct and in lifecycle order");
        }
        stages.push(Stage::ALL[ordinal]);
        previous = Some(ordinal);
    }
    Ok(stages)
}
fn configuration() -> (Selection, Vec<Stage>, Option<Config>) {
    let selection = Selection::parse(
        &required("SURGEIST_MEMORY_POPULATION"),
        &required("SURGEIST_MEMORY_MANAGED"),
        &required("SURGEIST_MEMORY_SAMPLE"),
    )
    .expect("explicit fixed native feature selection");
    let stages = checkpoints(&required("SURGEIST_MEMORY_HOLD_STAGE"))
        .expect("explicit feature holder checkpoints");
    let directory = optional("SURGEIST_MEMORY_STAGE_DIR");
    let token = optional("SURGEIST_MEMORY_TOKEN");
    let timeout = optional("SURGEIST_MEMORY_ACK_TIMEOUT_SECONDS");
    let config = checkpoint_config(
        !stages.is_empty(),
        directory.as_deref(),
        token.as_deref(),
        timeout.as_deref(),
    )
    .expect("explicit native feature checkpoint transport");
    (selection, stages, config)
}

#[test]
#[ignore = "explicit fresh-process release fixture; select native-features/13/sample and optional holder checkpoints"]
fn native_feature_selected_source_import_lifecycle() {
    require_release();
    let (selection, stages, config) = configuration();
    let mut gate = config
        .map(|config| Gate::open(config, stages.iter().map(|stage| stage.name()).collect()))
        .transpose()
        .expect("fresh runner-owned feature checkpoint directory");
    drop(stages);
    let pid = std::process::id();
    eprintln!(
        "native_features_selected profile=release pid={pid} sample={} source_nodes=10 target_nodes=3 managed=13 managed_means_combined_source_target_node_identities=true held_graph_identity_peak=13 controlled_owner_peak=10 style_sheet_genuine_cssom_publication=true style_outside_import=true form_history_empty_not_growable_by_public_front=true hosted_families_not_populated=true clock_enabled={} no_timing_slo=true observers_oracles_excluded_from_call_clocks=true batch_authoring_and_protocol_input_preparation_outside_clocks=true parser_interrupt_poll_inside_parse_call=true physical_backend=external_not_run_by_fixture all_zones_include_harness=true kernel_peak_since_launch=true logger_complete_process_peak_coverage=false",
        selection.sample.name(),
        !scale_metrics::ENABLED,
    );
    let times = run_sample(Some(&mut |at| {
        let held = at.holders();
        let source_owner_nodes = usize::from(held.source_owner) * SOURCE_NODES;
        let source_snapshot_nodes = usize::from(held.source_snapshots != 0) * SOURCE_NODES;
        let target_owner_nodes = if held.target_owner {
            if held.import_lease { 1 } else { TARGET_NODES }
        } else {
            0
        };
        let target_snapshot_nodes =
            usize::from(held.target_old || held.target_current) * TARGET_NODES;
        let import_captured_nodes = usize::from(held.import_lease) * SOURCE_NODES;
        eprintln!(
            "native_features_stage pid={pid} sample={} stage={} source_owner_live={} source_owner_nodes={source_owner_nodes} source_snapshots={} source_snapshot_node_identities={source_snapshot_nodes} target_owner_live={} target_published_nodes={target_owner_nodes} target_old_snapshot_live={} target_current_snapshot_live={} target_snapshot_node_identities={target_snapshot_nodes} import_lease_live={} import_captured_input_graph_nodes={import_captured_nodes} captured_input_aliases_external_source_current=true exact_arc_holder_count_not_enumerated=true local_cssom_handles_live=false external_input_collections_commits_requests_and_tickets_not_held=true counts_are_node_identities_not_allocations=true",
            selection.sample.name(),
            at.name(),
            held.source_owner,
            held.source_snapshots,
            held.target_owner,
            held.target_old,
            held.target_current,
            held.import_lease,
        );
        if let Some(gate) = &mut gate {
            gate.checkpoint(selection.identity(), pid, at.name())
                .unwrap_or_else(|error| panic!("feature checkpoint {} failed: {error}", at.name()));
        }
    }));
    if let Some(gate) = gate {
        assert!(
            gate.is_complete(),
            "all selected feature checkpoints acknowledged"
        );
    }
    eprintln!(
        "native_features_complete pid={pid} sample={} clock_enabled={} individual_call_times={times:?} allocator_reclamation_not_observed=true",
        selection.sample.name(),
        !scale_metrics::ENABLED
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feature_selection_requires_the_exact_small_geometry_and_ordered_stages() {
        for sample in ["warmup", "0", "1", "2"] {
            assert!(Selection::parse("native-features", "13", sample).is_ok());
        }
        for size in ["0", "10", "12", "14", "1000"] {
            assert!(Selection::parse("native-features", size, "0").is_err());
        }
        assert!(Selection::parse("all", "13", "0").is_err());
        assert!(Selection::parse("native-features", "13", "3").is_err());
        assert_eq!(checkpoints("all").unwrap(), Stage::ALL);
        assert!(checkpoints("none").unwrap().is_empty());
        assert_eq!(
            checkpoints("source-edited,import-begun,released").unwrap(),
            [Stage::SourceEdited, Stage::ImportBegun, Stage::Released]
        );
        for value in [
            "",
            "all,import-begun",
            "import-begun,import-begun",
            "released,source-edited",
        ] {
            assert!(checkpoints(value).is_err());
        }
    }
}
