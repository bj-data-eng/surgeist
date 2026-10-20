//! One explicitly selected stable-name lifecycle, using its existing owning
//! runner. The transport receives scalar counts only and retains no name keys.

use super::require_release;
use super::selection::{Sample, checkpoint_config, optional, required};
use crate::store::name_registry::{
    MemoryPopulation as Population, MemoryStage as Stage, NameMemoryObservation as Observation,
    run_memory_lifecycle,
};
use surgeist_test::checkpoint::{Config, Gate, Identity};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Selection {
    population: Population,
    count: usize,
    sample: Sample,
}

impl Selection {
    fn parse(pattern: &str, count: &str, sample: &str) -> Result<Self, &'static str> {
        let population = match pattern {
            "unique-text" => Population::UniqueText,
            "repeated-components" => Population::RepeatedComponents,
            _ => return Err("invalid SURGEIST_MEMORY_NAMES_PATTERN"),
        };
        let count = count
            .parse()
            .map_err(|_| "invalid SURGEIST_MEMORY_NAMES_COUNT")?;
        if ![32, 128].contains(&count) {
            return Err("unsupported SURGEIST_MEMORY_NAMES_COUNT");
        }
        Ok(Self {
            population,
            count,
            sample: Sample::parse(sample)?,
        })
    }
}

/// This closed identity discloses the held key container and accounting owner.
/// No nodes, snapshots, input vectors or diagnostic Weak observers are held.
#[derive(Clone, Copy, Debug)]
struct NamesIdentity {
    selection: Selection,
    at: Observation,
}

impl Identity for NamesIdentity {
    fn record(self, token: &str, pid: u32, stage: &str, ordinal: Option<usize>) -> String {
        assert_eq!(stage, self.at.stage.name());
        format!(
            "token={token}\npid={pid}\nfixture=stable-names\npattern={}\npopulation_size={}\nrequests={}\ndistinct={}\nmanaged=0\nkeys={}\nkey_capacity={}\nlive_names={}\ndead_rows={}\nowner_live={}\naccounting_live={}\nold_snapshot_live=false\ncurrent_snapshot_live=false\nsample={}\nstage={stage}\ncheckpoint={}\n",
            self.selection.population.name(),
            self.selection.count,
            self.selection
                .population
                .component_requests(self.selection.count),
            self.selection.population.distinct(self.selection.count),
            self.at.keys,
            self.at.key_capacity,
            self.at.live_names,
            self.at.dead_rows,
            self.at.stage.owner_live(),
            self.at.stage.accounting_live(),
            self.selection.sample.name(),
            ordinal.unwrap_or(0),
        )
    }
}

fn checkpoints(value: &str) -> Result<Vec<Stage>, &'static str> {
    if value == "none" {
        return Ok(Vec::new());
    }
    if value == "all" {
        return Ok(Stage::ALL.to_vec());
    }
    let mut selected = Vec::new();
    let mut previous = None;
    for name in value.split(',') {
        let ordinal = Stage::ALL
            .iter()
            .position(|stage| stage.name() == name)
            .ok_or("invalid stable-name checkpoint")?;
        if previous.is_some_and(|previous| ordinal <= previous) {
            return Err("stable-name checkpoints must be distinct and in lifecycle order");
        }
        selected.push(Stage::ALL[ordinal]);
        previous = Some(ordinal);
    }
    Ok(selected)
}

fn configuration() -> (Selection, Vec<Stage>, Option<Config>) {
    let selection = Selection::parse(
        &required("SURGEIST_MEMORY_NAMES_PATTERN"),
        &required("SURGEIST_MEMORY_NAMES_COUNT"),
        &required("SURGEIST_MEMORY_SAMPLE"),
    )
    .expect("explicit bounded stable-name selection");
    let stages = checkpoints(&required("SURGEIST_MEMORY_HOLD_STAGE"))
        .expect("explicit stable-name checkpoint selection");
    let directory = optional("SURGEIST_MEMORY_STAGE_DIR");
    let token = optional("SURGEIST_MEMORY_TOKEN");
    let timeout = optional("SURGEIST_MEMORY_ACK_TIMEOUT_SECONDS");
    let config = checkpoint_config(
        !stages.is_empty(),
        directory.as_deref(),
        token.as_deref(),
        timeout.as_deref(),
    )
    .expect("explicit stable-name checkpoint transport");
    (selection, stages, config)
}

#[test]
#[ignore = "explicit fresh-process release fixture; select one 32/128 name pattern/sample and optional lifecycle checkpoints"]
fn stable_name_selected_lifecycle() {
    require_release();
    let (selection, stages, config) = configuration();
    // Every selector and checkpoint is validated and the gate's directories
    // formed before the first harness-only observation. hold=none performs no IO.
    let mut gate = config
        .map(|config| Gate::open(config, stages.iter().map(|stage| stage.name()).collect()))
        .transpose()
        .expect("fresh runner-owned stable-name stage directory");
    drop(stages);
    let pid = std::process::id();
    eprintln!(
        "name_selected profile=release pid={pid} pattern={} population_size={} requests={} distinct={} sample={} managed=0 snapshots=0 mandatory_identity_only=true optional_pool_not_entered=true inputs_expected_sets_diagnostic_weaks_dropped_before_holds=true key_container_capacity_disclosed=true accounting_survives_owner_until_final_key_release=true physical_backend=external_not_run_by_fixture all_zones_include_harness=true kernel_peak_since_launch=true logger_complete_process_peak_coverage=false",
        selection.population.name(),
        selection.count,
        selection.population.component_requests(selection.count),
        selection.population.distinct(selection.count),
        selection.sample.name(),
    );
    run_memory_lifecycle(
        selection.population,
        selection.count,
        false,
        Some(&mut |at| {
            eprintln!(
                "name_selected_stage pid={pid} pattern={} population_size={} sample={} stage={} live_names={} dead_rows={} keys={} key_capacity={} owner_live={} accounting_live={} managed=0 old_snapshot_live=false current_snapshot_live=false",
                selection.population.name(),
                selection.count,
                selection.sample.name(),
                at.stage.name(),
                at.live_names,
                at.dead_rows,
                at.keys,
                at.key_capacity,
                at.stage.owner_live(),
                at.stage.accounting_live(),
            );
            if let Some(gate) = &mut gate {
                gate.checkpoint(NamesIdentity { selection, at }, pid, at.stage.name())
                    .unwrap_or_else(|error| {
                        panic!("stable-name stage {} failed: {error}", at.stage.name())
                    });
            }
        }),
    );
    if let Some(gate) = gate {
        assert!(
            gate.is_complete(),
            "all selected name checkpoints acknowledged"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_selection_is_bounded_and_checkpoint_order_is_explicit() {
        for pattern in ["unique-text", "repeated-components"] {
            for count in ["32", "128"] {
                assert!(Selection::parse(pattern, count, "0").is_ok());
            }
        }
        for count in ["0", "31", "129", "1000", "-1"] {
            assert!(Selection::parse("unique-text", count, "0").is_err());
        }
        assert!(Selection::parse("all", "32", "0").is_err());
        assert!(Selection::parse("unique-text", "32", "3").is_err());
        assert!(checkpoints("none").unwrap().is_empty());
        assert_eq!(checkpoints("all").unwrap(), Stage::ALL);
        assert_eq!(
            checkpoints("all-keys-held,owner-released-key-held,last-key-released").unwrap(),
            [
                Stage::AllKeysHeld,
                Stage::OwnerReleased,
                Stage::LastKeyReleased
            ]
        );
        for value in [
            "",
            "none,all-keys-held",
            "all-keys-held,all-keys-held",
            "last-key-released,all-keys-held",
        ] {
            assert!(checkpoints(value).is_err());
        }
    }

    #[test]
    fn name_identity_binds_scalar_selection_and_holders_within_transport_limit() {
        let selection = Selection::parse("repeated-components", "128", "warmup").unwrap();
        let identity = NamesIdentity {
            selection,
            at: Observation {
                stage: Stage::DeadRowsBeforeCleanup,
                live_names: 1,
                dead_rows: 4,
                keys: 1,
                key_capacity: 3,
            },
        };
        let token = "n".repeat(64);
        let record = identity.record(&token, u32::MAX, "dead-rows-before-cleanup", Some(3));
        assert_eq!(
            record,
            format!(
                "token={token}\npid=4294967295\nfixture=stable-names\npattern=repeated-components\npopulation_size=128\nrequests=256\ndistinct=5\nmanaged=0\nkeys=1\nkey_capacity=3\nlive_names=1\ndead_rows=4\nowner_live=true\naccounting_live=true\nold_snapshot_live=false\ncurrent_snapshot_live=false\nsample=warmup\nstage=dead-rows-before-cleanup\ncheckpoint=3\n"
            )
        );
        assert!(record.len() <= 512);
    }
}
