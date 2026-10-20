//! A fixed real parser-report/DOM fixture in one fresh process. Optional
//! checkpoints expose scalar identity only; the runner owns external tools.

use super::super::{
    require_release, scale_metrics,
    selection::{Identity, Sample, checkpoint_config, optional, required},
};
use super::{CELLS, COLUMNS, HolderStage, MANAGED_NODES, PARSED_NODES, ROWS, run_sample};
use surgeist_test::checkpoint::{Config, Gate};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Selection {
    sample: Sample,
}

impl Selection {
    fn parse(
        population: &str,
        rows: &str,
        columns: &str,
        managed: &str,
        sample: &str,
    ) -> Result<Self, &'static str> {
        if population != "parsed-table" {
            return Err("SURGEIST_MEMORY_POPULATION must be parsed-table");
        }
        let rows: usize = rows.parse().map_err(|_| "invalid SURGEIST_MEMORY_ROWS")?;
        let columns: usize = columns
            .parse()
            .map_err(|_| "invalid SURGEIST_MEMORY_COLUMNS")?;
        let managed: usize = managed
            .parse()
            .map_err(|_| "invalid SURGEIST_MEMORY_MANAGED")?;
        if (rows, columns, managed) != (ROWS, COLUMNS, MANAGED_NODES) {
            return Err(
                "the parsed fixture requires exactly 4 rows, 8 columns and 76 managed nodes",
            );
        }
        Ok(Self {
            sample: Sample::parse(sample)?,
        })
    }

    fn identity(self) -> Identity {
        Identity::Parsed {
            rows: ROWS,
            columns: COLUMNS,
            cells: CELLS,
            parsed_nodes: PARSED_NODES,
            managed: MANAGED_NODES,
            sample: self.sample,
        }
    }
}

fn checkpoints(value: &str) -> Result<Vec<HolderStage>, &'static str> {
    match value {
        "none" => return Ok(Vec::new()),
        "all" => return Ok(HolderStage::ALL.to_vec()),
        _ => {}
    }
    let mut result = Vec::new();
    let mut previous = None;
    for name in value.split(',') {
        let index = HolderStage::ALL
            .iter()
            .position(|stage| stage.name() == name)
            .ok_or("invalid parsed SURGEIST_MEMORY_HOLD_STAGE")?;
        if previous.is_some_and(|previous| index <= previous) {
            return Err("parsed checkpoints must be distinct and in lifecycle order");
        }
        previous = Some(index);
        result.push(HolderStage::ALL[index]);
    }
    Ok(result)
}

fn configuration() -> (Selection, Vec<HolderStage>, Option<Config>) {
    let selected = Selection::parse(
        &required("SURGEIST_MEMORY_POPULATION"),
        &required("SURGEIST_MEMORY_ROWS"),
        &required("SURGEIST_MEMORY_COLUMNS"),
        &required("SURGEIST_MEMORY_MANAGED"),
        &required("SURGEIST_MEMORY_SAMPLE"),
    )
    .expect("explicit fixed parsed memory selection");
    if let Some(existing) = optional("SURGEIST_SCALE_SIZES") {
        assert_eq!(
            existing,
            MANAGED_NODES.to_string(),
            "conflicting inherited scale size selection"
        );
    }
    let stages = checkpoints(&required("SURGEIST_MEMORY_HOLD_STAGE"))
        .expect("explicit parsed checkpoint selection");
    let directory = optional("SURGEIST_MEMORY_STAGE_DIR");
    let token = optional("SURGEIST_MEMORY_TOKEN");
    let timeout = optional("SURGEIST_MEMORY_ACK_TIMEOUT_SECONDS");
    let config = checkpoint_config(
        !stages.is_empty(),
        directory.as_deref(),
        token.as_deref(),
        timeout.as_deref(),
    )
    .expect("explicit parsed stage gate configuration");
    (selected, stages, config)
}

fn holders(stage: HolderStage) -> (bool, bool, bool, bool) {
    match stage {
        HolderStage::HarnessOnly | HolderStage::Released => (false, false, false, false),
        HolderStage::ParsedReport | HolderStage::ReportOnly => (false, true, false, false),
        HolderStage::Populated
        | HolderStage::BeforeTextEdit
        | HolderStage::TextEdited
        | HolderStage::BeforeClassEdit
        | HolderStage::ClassEdited
        | HolderStage::BeforeStyleEdit
        | HolderStage::StyleEdited => (true, true, true, false),
        HolderStage::Edited => (true, true, true, true),
        HolderStage::OwnerDropped => (false, true, true, true),
        HolderStage::OldOnly => (false, true, true, false),
    }
}

#[test]
#[ignore = "explicit fresh-process release fixture; select the fixed parsed table/sample and optional ordered holder checkpoints"]
fn parsed_selected_report_holder_stages() {
    require_release();
    // Reject all inherited/invalid selectors and acknowledgement configuration
    // before parsing, forming a DOM, or touching the runner directory.
    let (selected, stages, config) = configuration();
    let stage_names = stages.iter().map(|stage| stage.name()).collect();
    drop(stages);
    let mut gate = config
        .map(|config| Gate::open(config, stage_names))
        .transpose()
        .expect("fresh runner-owned stage directory");
    let pid = std::process::id();
    eprintln!(
        "memory_parsed_selected profile=release pid={pid} population=parsed-table rows={ROWS} columns={COLUMNS} cells={CELLS} parsed_nodes={PARSED_NODES} managed={MANAGED_NODES} sample={} counters_enabled={} counters_build_value={:?} revision={} toolchain={} machine={} os={} arch={} array_inventory=false physical_backend=external_not_run_by_fixture timings_not_reported=true mutation_peak_not_established=true",
        selected.sample.name(),
        scale_metrics::ENABLED,
        option_env!("SURGEIST_DOM_STORAGE_COUNTERS"),
        optional("SURGEIST_SCALE_REVISION").unwrap_or_else(|| "unknown".into()),
        optional("SURGEIST_SCALE_TOOLCHAIN").unwrap_or_else(|| "unknown".into()),
        optional("SURGEIST_SCALE_MACHINE").unwrap_or_else(|| "unknown".into()),
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    let mut observe = |at| {
        let (owner, report, old, current) = holders(at);
        let source_ids = !matches!(
            at,
            HolderStage::HarnessOnly | HolderStage::ReportOnly | HolderStage::Released
        );
        let cell_ids = matches!(
            at,
            HolderStage::Populated
                | HolderStage::BeforeTextEdit
                | HolderStage::TextEdited
                | HolderStage::BeforeClassEdit
                | HolderStage::ClassEdited
                | HolderStage::BeforeStyleEdit
                | HolderStage::StyleEdited
                | HolderStage::Edited
                | HolderStage::OwnerDropped
        );
        let wrapper_weak = !matches!(at, HolderStage::HarnessOnly | HolderStage::Released);
        eprintln!(
            "memory_parsed_selected_stage pid={pid} population=parsed-table rows={ROWS} columns={COLUMNS} cells={CELLS} parsed_nodes={PARSED_NODES} managed={MANAGED_NODES} sample={} stage={} owner_live={owner} report_live={report} old_snapshot_live={old} current_snapshot_live={current} source_and_source_ids_live={source_ids} cell_text_id_scratch_live={cell_ids} report_wrapper_weak_live={wrapper_weak} instrumentation_storage_not_subtracted=true allocator_reclamation_not_observed=true",
            selected.sample.name(),
            at.name()
        );
        if let Some(gate) = &mut gate {
            gate.checkpoint(selected.identity(), pid, at.name())
                .unwrap_or_else(|error| panic!("parsed stage {} failed: {error}", at.name()));
        }
    };
    assert!(run_sample(false, Some(&mut observe), false).is_none());
    if let Some(gate) = gate {
        assert!(
            gate.is_complete(),
            "every selected parsed checkpoint was acknowledged"
        );
    }
    eprintln!(
        "memory_parsed_selected_complete pid={pid} sample={} physical_total_not_inferred=true",
        selected.sample.name()
    );
}

#[test]
#[ignore = "explicit fresh-process release timing fixture; fixed parsed table/sample, hold=none and counters disabled"]
fn parsed_selected_operation_timings() {
    require_release();
    assert!(
        scale_metrics::require_enabled().is_err(),
        "operation timings require a build without storage counters"
    );
    let (selected, stages, config) = configuration();
    assert!(
        stages.is_empty() && config.is_none(),
        "operation timings require SURGEIST_MEMORY_HOLD_STAGE=none and no gate transport"
    );
    drop(stages);
    drop(config);
    // One fixed lifecycle, no observer or array inventory. All output below
    // follows the actual owner, snapshot, report and report-Weak disposal.
    let times = run_sample(false, None, true).expect("normal-build operation clock");
    let pid = std::process::id();
    eprintln!(
        "memory_parsed_operation_timings profile=release pid={pid} population=parsed-table rows={ROWS} columns={COLUMNS} cells={CELLS} parsed_nodes={PARSED_NODES} managed={MANAGED_NODES} sample={} revision={} toolchain={} machine={} os={} arch={} units=ns parse={} owner_environment_setup={} installation={} initial_snapshot_clone={} final_snapshot={} text_edit={} class_edit={} style_edit={} installation_result_drop={} owner_drop={} current_snapshot_drop={} old_snapshot_drop={} report_drop={} report_weak_drop={} counters_enabled=false gates=false array_inventory=false output_after_release=true setup_includes_origin_batch_and_environment=true request_construction_and_report_Arc_wrapping_timed=false edit_batch_formation_and_commit_disposal_timed=false interoperation_oracles_present=true zero_readings_preserved=true clock_overhead_subtracted=false timing_thresholds=false unperturbed_or_physical_total_claim=false",
        selected.sample.name(),
        optional("SURGEIST_SCALE_REVISION").unwrap_or_else(|| "unknown".into()),
        optional("SURGEIST_SCALE_TOOLCHAIN").unwrap_or_else(|| "unknown".into()),
        optional("SURGEIST_SCALE_MACHINE").unwrap_or_else(|| "unknown".into()),
        std::env::consts::OS,
        std::env::consts::ARCH,
        times.parse.as_nanos(),
        times.owner_environment_setup.as_nanos(),
        times.installation.as_nanos(),
        times.initial_snapshot_clone.as_nanos(),
        times.final_snapshot.as_nanos(),
        times.text_edit.as_nanos(),
        times.class_edit.as_nanos(),
        times.style_edit.as_nanos(),
        times.installation_result_drop.as_nanos(),
        times.owner_drop.as_nanos(),
        times.current_snapshot_drop.as_nanos(),
        times.old_snapshot_drop.as_nanos(),
        times.report_drop.as_nanos(),
        times.report_weak_drop.as_nanos(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_parsed_selection_refuses_other_geometry_and_samples() {
        assert!(Selection::parse("parsed-table", "4", "8", "76", "0").is_ok());
        assert!(Selection::parse("parsed-table", "4", "8", "76", "warmup").is_ok());
        for (population, rows, columns, managed, sample) in [
            ("detached-elements", "4", "8", "76", "0"),
            ("parsed-table", "5", "8", "76", "0"),
            ("parsed-table", "4", "9", "76", "0"),
            ("parsed-table", "4", "8", "75", "0"),
            ("parsed-table", "4", "8", "76", "3"),
        ] {
            assert!(Selection::parse(population, rows, columns, managed, sample).is_err());
        }
    }

    #[test]
    fn parsed_checkpoint_selection_requires_lifecycle_order_without_duplicates() {
        assert!(checkpoints("none").unwrap().is_empty());
        assert_eq!(checkpoints("all").unwrap(), HolderStage::ALL);
        assert_eq!(
            checkpoints("populated,edited,report-only,released").unwrap(),
            [
                HolderStage::Populated,
                HolderStage::Edited,
                HolderStage::ReportOnly,
                HolderStage::Released,
            ]
        );
        for invalid in [
            "",
            "all,edited",
            "edited,edited",
            "released,populated",
            "edited,",
            "native-only",
        ] {
            assert!(checkpoints(invalid).is_err());
        }
    }

    #[test]
    fn external_inventory_disabled_path_still_checks_the_complete_real_lifecycle() {
        let mut stages = Vec::new();
        assert!(run_sample(false, Some(&mut |stage| stages.push(stage)), false).is_none());
        assert_eq!(
            stages,
            [
                HolderStage::HarnessOnly,
                HolderStage::ParsedReport,
                HolderStage::Populated,
                HolderStage::BeforeTextEdit,
                HolderStage::TextEdited,
                HolderStage::BeforeClassEdit,
                HolderStage::ClassEdited,
                HolderStage::BeforeStyleEdit,
                HolderStage::StyleEdited,
                HolderStage::Edited,
                HolderStage::OwnerDropped,
                HolderStage::OldOnly,
                HolderStage::ReportOnly,
                HolderStage::Released,
            ]
        );
    }
}
