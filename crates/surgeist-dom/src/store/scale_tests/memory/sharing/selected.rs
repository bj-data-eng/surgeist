//! One fixed native layered workload per process, using the same ordinary
//! construction, literal, topology, backing and isolated-edit checks.

use super::super::{
    require_release, scale_metrics,
    selection::{Identity, Sample, checkpoint_config, optional, required},
};
use super::{CELLS, COLUMNS, HolderStage, Mode, NODES, ROWS, Unique, run_sample};
use surgeist_test::checkpoint::{Config, Gate};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Selection {
    mode: Mode,
    unique: Unique,
    sample: Sample,
}

impl Selection {
    fn parse(
        population: &str,
        rows: &str,
        columns: &str,
        managed: &str,
        mode: &str,
        ids: &str,
        sample: &str,
    ) -> Result<Self, &'static str> {
        if population != "native-layered" {
            return Err("SURGEIST_MEMORY_POPULATION must be native-layered");
        }
        let rows: usize = rows.parse().map_err(|_| "invalid SURGEIST_MEMORY_ROWS")?;
        let columns: usize = columns
            .parse()
            .map_err(|_| "invalid SURGEIST_MEMORY_COLUMNS")?;
        let managed: usize = managed
            .parse()
            .map_err(|_| "invalid SURGEIST_MEMORY_MANAGED")?;
        if (rows, columns, managed) != (ROWS, COLUMNS, NODES) {
            return Err(
                "the layered fixture requires exactly 2 rows, 3 columns and 16 managed nodes",
            );
        }
        let mode = match mode {
            "independent" => Mode::Independent,
            "caller-shared" => Mode::CallerShared,
            "banks-only" => Mode::BanksOnly,
            "combined" => Mode::Combined,
            _ => return Err("invalid SURGEIST_MEMORY_MODE"),
        };
        let unique = match ids {
            "repeated" => Unique::None,
            "row-ids" => Unique::Rows,
            "cell-ids" => Unique::Cells,
            _ => return Err("invalid SURGEIST_MEMORY_IDS"),
        };
        Ok(Self {
            mode,
            unique,
            sample: Sample::parse(sample)?,
        })
    }

    fn identity(self) -> Identity {
        Identity::Layered {
            mode: self.mode.name(),
            ids: self.unique.name(),
            rows: ROWS,
            columns: COLUMNS,
            cells: CELLS,
            managed: NODES,
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
            .ok_or("invalid layered SURGEIST_MEMORY_HOLD_STAGE")?;
        if previous.is_some_and(|previous| index <= previous) {
            return Err("layered checkpoints must be distinct and in lifecycle order");
        }
        previous = Some(index);
        result.push(HolderStage::ALL[index]);
    }
    Ok(result)
}

fn configuration() -> (Selection, Vec<HolderStage>, Option<Config>) {
    let selection = Selection::parse(
        &required("SURGEIST_MEMORY_POPULATION"),
        &required("SURGEIST_MEMORY_ROWS"),
        &required("SURGEIST_MEMORY_COLUMNS"),
        &required("SURGEIST_MEMORY_MANAGED"),
        &required("SURGEIST_MEMORY_MODE"),
        &required("SURGEIST_MEMORY_IDS"),
        &required("SURGEIST_MEMORY_SAMPLE"),
    )
    .expect("explicit fixed native layered selection");
    if let Some(existing) = optional("SURGEIST_SCALE_SIZES") {
        assert_eq!(
            existing,
            NODES.to_string(),
            "conflicting inherited scale size selection"
        );
    }
    let stages = checkpoints(&required("SURGEIST_MEMORY_HOLD_STAGE"))
        .expect("explicit layered checkpoint selection");
    let directory = optional("SURGEIST_MEMORY_STAGE_DIR");
    let token = optional("SURGEIST_MEMORY_TOKEN");
    let timeout = optional("SURGEIST_MEMORY_ACK_TIMEOUT_SECONDS");
    let config = checkpoint_config(
        !stages.is_empty(),
        directory.as_deref(),
        token.as_deref(),
        timeout.as_deref(),
    )
    .expect("explicit layered stage gate configuration");
    (selection, stages, config)
}

struct Holders {
    owner: bool,
    old: bool,
    current: bool,
    inputs: bool,
    guard: bool,
    ids: bool,
}

fn holders(stage: HolderStage) -> Holders {
    let owner = matches!(
        stage,
        HolderStage::CallerInputsHeld
            | HolderStage::Populated
            | HolderStage::BeforeAttributeEdit
            | HolderStage::AttributeEdited
            | HolderStage::BeforeTextEdit
            | HolderStage::TextEdited
            | HolderStage::Edited
    );
    Holders {
        owner,
        old: !matches!(stage, HolderStage::HarnessOnly | HolderStage::Released),
        current: matches!(stage, HolderStage::Edited | HolderStage::OwnerDropped),
        inputs: stage == HolderStage::CallerInputsHeld,
        guard: owner,
        ids: !matches!(stage, HolderStage::HarnessOnly | HolderStage::Released),
    }
}

#[test]
#[ignore = "explicit fresh-process release fixture; select one fixed 2x3 layered mode/ID pattern/sample and optional checkpoints"]
fn native_layered_selected_holder_stages() {
    require_release();
    // Every selector and the full checkpoint plan is checked before any DOM
    // allocation or runner-directory access. The gate holds scalars only.
    let (selection, stages, config) = configuration();
    let stage_names = stages.iter().map(|stage| stage.name()).collect();
    drop(stages);
    let mut gate = config
        .map(|config| Gate::open(config, stage_names))
        .transpose()
        .expect("fresh runner-owned stage directory");
    let stage_gate = gate.is_some();
    let pid = std::process::id();
    eprintln!(
        "memory_layered_selected profile=release pid={pid} population=native-layered rows={ROWS} columns={COLUMNS} cells={CELLS} managed={NODES} mode={} ids={} sample={} counters_enabled={} counters_build_value={:?} revision={} toolchain={} machine={} os={} arch={} stage_gate_enabled={stage_gate} pointer_inventory=false physical_backend=external_not_run_by_fixture timings_exclude_callbacks=true setup_includes_inputs_batch_apply_identity_lookup=true setup_excludes_oracles=true edit_intervals_include_batch_apply=true edit_intervals_exclude_oracles_commit_drop=true tool_inspected_runs_are_perturbed=true mutation_peak_not_established=true",
        selection.mode.name(),
        selection.unique.name(),
        selection.sample.name(),
        scale_metrics::ENABLED,
        option_env!("SURGEIST_DOM_STORAGE_COUNTERS"),
        optional("SURGEIST_SCALE_REVISION").unwrap_or_else(|| "unknown".into()),
        optional("SURGEIST_SCALE_TOOLCHAIN").unwrap_or_else(|| "unknown".into()),
        optional("SURGEIST_SCALE_MACHINE").unwrap_or_else(|| "unknown".into()),
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    let mut observe = |at| {
        let held = holders(at);
        eprintln!(
            "memory_layered_selected_stage pid={pid} population=native-layered rows={ROWS} columns={COLUMNS} cells={CELLS} managed={NODES} mode={} ids={} sample={} stage={} owner_live={} old_snapshot_live={} current_snapshot_live={} caller_inputs_live={} control_weak_live={} creation_commit_live=false edit_commit_live=false id_scratch_live={} instrumentation_storage_not_subtracted=true allocator_reclamation_not_observed=true",
            selection.mode.name(),
            selection.unique.name(),
            selection.sample.name(),
            at.name(),
            held.owner,
            held.old,
            held.current,
            held.inputs,
            held.guard,
            held.ids
        );
        if let Some(gate) = &mut gate {
            gate.checkpoint(selection.identity(), pid, at.name())
                .unwrap_or_else(|error| panic!("layered stage {} failed: {error}", at.name()));
        }
    };
    let times = run_sample(selection.mode, selection.unique, Some(&mut observe));
    if let Some(gate) = gate {
        assert!(
            gate.is_complete(),
            "every selected layered checkpoint was acknowledged"
        );
    }
    if let Some(times) = times {
        eprintln!(
            "memory_layered_times pid={pid} mode={} ids={} sample={} counters_enabled=false stage_gate_enabled={stage_gate} setup_ns={} old_snapshot_ns={} caller_inputs_drop_ns={} attribute_edit_ns={} text_edit_ns={} current_snapshot_ns={} grid_and_backing_checks_ns={} owner_drop_ns={} current_drop_ns={} last_snapshot_drop_ns={} callback_waits_excluded=true tool_inspected_runs_are_perturbed=true unperturbed_execution_not_established=true",
            selection.mode.name(),
            selection.unique.name(),
            selection.sample.name(),
            times.setup.as_nanos(),
            times.old_snapshot.as_nanos(),
            times.caller_inputs_drop.as_nanos(),
            times.attribute_edit.as_nanos(),
            times.text_edit.as_nanos(),
            times.current_snapshot.as_nanos(),
            times.oracle_checks.as_nanos(),
            times.owner_drop.as_nanos(),
            times.current_drop.as_nanos(),
            times.last_snapshot_drop.as_nanos()
        );
    } else {
        eprintln!("memory_layered_times pid={pid} omitted=compiled_counters_enabled");
    }
    eprintln!(
        "memory_layered_selected_complete pid={pid} mode={} ids={} sample={} physical_total_not_inferred=true",
        selection.mode.name(),
        selection.unique.name(),
        selection.sample.name()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_layered_selection_accepts_only_the_twelve_named_cases() {
        for (mode_name, mode) in [
            ("independent", Mode::Independent),
            ("caller-shared", Mode::CallerShared),
            ("banks-only", Mode::BanksOnly),
            ("combined", Mode::Combined),
        ] {
            for (ids, unique) in [
                ("repeated", Unique::None),
                ("row-ids", Unique::Rows),
                ("cell-ids", Unique::Cells),
            ] {
                assert_eq!(
                    Selection::parse("native-layered", "2", "3", "16", mode_name, ids, "0")
                        .unwrap(),
                    Selection {
                        mode,
                        unique,
                        sample: Sample::First
                    }
                );
            }
        }
        for values in [
            ["parsed-table", "2", "3", "16", "combined", "repeated", "0"],
            [
                "native-layered",
                "3",
                "3",
                "16",
                "combined",
                "repeated",
                "0",
            ],
            [
                "native-layered",
                "2",
                "4",
                "16",
                "combined",
                "repeated",
                "0",
            ],
            [
                "native-layered",
                "2",
                "3",
                "1000",
                "combined",
                "repeated",
                "0",
            ],
            ["native-layered", "2", "3", "16", "all", "repeated", "0"],
            ["native-layered", "2", "3", "16", "combined", "all", "0"],
            [
                "native-layered",
                "2",
                "3",
                "16",
                "combined",
                "repeated",
                "3",
            ],
        ] {
            assert!(
                Selection::parse(
                    values[0], values[1], values[2], values[3], values[4], values[5], values[6]
                )
                .is_err()
            );
        }
    }

    #[test]
    fn layered_checkpoints_require_distinct_lifecycle_order() {
        assert!(checkpoints("none").unwrap().is_empty());
        assert_eq!(checkpoints("all").unwrap().len(), 11);
        assert_eq!(
            checkpoints("caller-inputs-held,populated,owner-dropped,released").unwrap(),
            [
                HolderStage::CallerInputsHeld,
                HolderStage::Populated,
                HolderStage::OwnerDropped,
                HolderStage::Released,
            ]
        );
        for value in [
            "",
            "all,edited",
            "edited,edited",
            "released,populated",
            "populated,",
            "report-only",
        ] {
            assert!(checkpoints(value).is_err());
        }
    }

    #[test]
    fn all_layer_modes_and_id_patterns_preserve_literals_and_release_stages() {
        for mode in [
            Mode::Independent,
            Mode::CallerShared,
            Mode::BanksOnly,
            Mode::Combined,
        ] {
            for unique in [Unique::None, Unique::Rows, Unique::Cells] {
                let mut stages = Vec::new();
                let times = run_sample(mode, unique, Some(&mut |stage| stages.push(stage)));
                assert_eq!(times.is_some(), !scale_metrics::ENABLED);
                assert_eq!(
                    stages,
                    [
                        HolderStage::HarnessOnly,
                        HolderStage::CallerInputsHeld,
                        HolderStage::Populated,
                        HolderStage::BeforeAttributeEdit,
                        HolderStage::AttributeEdited,
                        HolderStage::BeforeTextEdit,
                        HolderStage::TextEdited,
                        HolderStage::Edited,
                        HolderStage::OwnerDropped,
                        HolderStage::OldOnly,
                        HolderStage::Released,
                    ],
                    "{mode:?} {unique:?}"
                );
            }
        }
    }
}
