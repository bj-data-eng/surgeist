//! One explicitly selected native population/sample per process. The optional
//! single-stage gate lends no DOM authority and never owns a semantic holder.
//! The runner owns its fresh directory and any external measurement process.

use super::gate::{self, Gate, Identity, Sample, optional, required};
use super::{
    HolderStage, Population, construction_peak, require_release, run_sample, scale_metrics,
};

const SIZES: [usize; 5] = [1000, 10_000, 100_000, 800_000, 1_000_000];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Selection {
    population: Population,
    managed: usize,
    sample: Sample,
}

impl Selection {
    fn parse(population: &str, managed: &str, sample: &str) -> Result<Self, &'static str> {
        let population = match population {
            "detached-elements" => Population::DetachedElements,
            "detached-text" => Population::DetachedText,
            "attached-elements" => Population::AttachedElements,
            "attached-text" => Population::AttachedText,
            "vacant-comments" => Population::VacantComments,
            _ => return Err("invalid SURGEIST_MEMORY_POPULATION"),
        };
        let managed: usize = managed
            .parse()
            .map_err(|_| "invalid SURGEIST_MEMORY_MANAGED")?;
        if !SIZES.contains(&managed) {
            return Err("unsupported SURGEIST_MEMORY_MANAGED");
        }
        construction_peak(population, managed)?;
        Ok(Self {
            population,
            managed,
            sample: Sample::parse(sample)?,
        })
    }

    fn identity(self) -> Identity {
        Identity::Native {
            population: self.population.name(),
            managed: self.managed,
            sample: self.sample,
        }
    }
}

fn stage(value: &str) -> Result<HolderStage, &'static str> {
    match value {
        "harness-only" => Ok(HolderStage::HarnessOnly),
        "populated" => Ok(HolderStage::Populated),
        "edited" => Ok(HolderStage::Edited),
        "owner-dropped" => Ok(HolderStage::OwnerDropped),
        "old-only" => Ok(HolderStage::OldOnly),
        "released" => Ok(HolderStage::Released),
        _ => Err("invalid SURGEIST_MEMORY_HOLD_STAGE"),
    }
}

#[derive(Debug)]
struct GateConfig {
    stage: HolderStage,
    transport: gate::Config,
}

impl GateConfig {
    fn parse(
        hold: &str,
        directory: Option<&str>,
        token: Option<&str>,
        timeout: Option<&str>,
    ) -> Result<Option<Self>, &'static str> {
        if hold == "none" {
            gate::Config::parse(false, directory, token, timeout)?;
            return Ok(None);
        }
        let stage = stage(hold)?;
        let transport = gate::Config::parse(true, directory, token, timeout)?
            .expect("enabled gate configuration");
        Ok(Some(Self { stage, transport }))
    }
}

fn configuration() -> (Selection, Option<GateConfig>) {
    let selection = Selection::parse(
        &required("SURGEIST_MEMORY_POPULATION"),
        &required("SURGEIST_MEMORY_MANAGED"),
        &required("SURGEIST_MEMORY_SAMPLE"),
    )
    .expect("explicit native memory selection");
    if let Some(existing) = optional("SURGEIST_SCALE_SIZES") {
        assert_eq!(
            existing,
            selection.managed.to_string(),
            "conflicting inherited scale size selection"
        );
    }
    let hold = required("SURGEIST_MEMORY_HOLD_STAGE");
    let directory = optional("SURGEIST_MEMORY_STAGE_DIR");
    let token = optional("SURGEIST_MEMORY_TOKEN");
    let timeout = optional("SURGEIST_MEMORY_ACK_TIMEOUT_SECONDS");
    let gate = GateConfig::parse(
        &hold,
        directory.as_deref(),
        token.as_deref(),
        timeout.as_deref(),
    )
    .expect("explicit stage gate configuration");
    (selection, gate)
}

fn holders(stage: HolderStage) -> (bool, bool, bool) {
    match stage {
        HolderStage::HarnessOnly | HolderStage::Released => (false, false, false),
        HolderStage::Populated => (true, false, false),
        HolderStage::Edited => (true, true, true),
        HolderStage::OwnerDropped => (false, true, true),
        HolderStage::OldOnly => (false, true, false),
    }
}

#[test]
#[ignore = "explicit fresh-process release fixture; select one population/size/sample and optional holder stage"]
fn native_selected_population_holder_stages() {
    require_release();
    let (selected, config) = configuration();
    // All selectors and the construction peak are checked before run_sample
    // can allocate any DOM. hold=none never calls Gate::open or filesystem IO.
    let mut gate = config
        .map(|config| Gate::open(config.transport, vec![config.stage.name()]))
        .transpose()
        .expect("fresh runner-owned stage directory");
    let pid = std::process::id();
    let peak = construction_peak(selected.population, selected.managed).unwrap();
    eprintln!(
        "memory_selected profile=release pid={pid} population={} managed={} peak_managed={peak} sample={} counters_enabled={} counters_build_value={:?} revision={} toolchain={} machine={} os={} arch={} physical_backend=external_not_run_by_fixture waits_excluded_from_timings=true",
        selected.population.name(),
        selected.managed,
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
        let (owner, old, current) = holders(at);
        eprintln!(
            "memory_selected_stage pid={pid} population={} managed={} peak_managed={peak} sample={} stage={} owner_live={owner} old_snapshot_live={old} current_snapshot_live={current} rows=0 cells=0",
            selected.population.name(),
            selected.managed,
            selected.sample.name(),
            at.name()
        );
        if let Some(gate) = &mut gate {
            gate.checkpoint(selected.identity(), pid, at.name())
                .unwrap_or_else(|error| panic!("selected stage {} failed: {error}", at.name()));
        }
    };
    let times = run_sample(
        selected.population,
        selected.managed,
        selected.sample.ordinal(),
        matches!(selected.sample, Sample::Warmup),
        Some(&mut observe),
    );
    if let Some(gate) = gate {
        assert!(
            gate.is_complete(),
            "selected holder stage was reached and acknowledged"
        );
    }
    eprintln!(
        "memory_selected_complete pid={pid} population={} managed={} sample={} timings={times:?} allocator_reclamation_not_observed=true",
        selected.population.name(),
        selected.managed,
        selected.sample.name()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_selectors_refuse_invalid_values_and_over_ceiling_construction() {
        for name in [
            "detached-elements",
            "detached-text",
            "attached-elements",
            "attached-text",
            "vacant-comments",
        ] {
            assert_eq!(
                Selection::parse(name, "1000", "warmup").unwrap().managed,
                1000
            );
        }
        assert!(Selection::parse("detached-elements", "1000000", "2").is_ok());
        assert!(Selection::parse("vacant-comments", "1000000", "0").is_err());
        let vacant = Selection::parse("vacant-comments", "800000", "0").unwrap();
        assert_eq!(
            construction_peak(vacant.population, vacant.managed),
            Ok(1_000_000)
        );
        assert!(construction_peak(Population::DetachedText, usize::MAX).is_err());
        assert!(Selection::parse("all", "1000", "0").is_err());
        assert!(Selection::parse("detached-text", "2000000", "0").is_err());
        assert!(Selection::parse("detached-text", "1000", "3").is_err());
        assert!(
            GateConfig::parse("none", None, None, None)
                .unwrap()
                .is_none()
        );
        assert!(GateConfig::parse("none", Some("not-accessed"), None, None).is_err());
        assert!(
            GateConfig::parse(
                "edited",
                Some("not-accessed"),
                Some("bad\ntoken"),
                Some("1")
            )
            .is_err()
        );
        assert!(
            GateConfig::parse("edited", Some("not-accessed"), Some("token"), Some("0")).is_err()
        );
        assert!(
            GateConfig::parse("edited", Some("not-accessed"), Some("token"), Some("301")).is_err()
        );
    }

    #[test]
    fn one_real_native_sample_emits_checked_stages_and_releases_its_holders() {
        let mut stages = Vec::new();
        run_sample(
            Population::DetachedText,
            1000,
            0,
            false,
            Some(&mut |stage| stages.push(stage)),
        );
        assert_eq!(
            stages,
            [
                HolderStage::HarnessOnly,
                HolderStage::Populated,
                HolderStage::Edited,
                HolderStage::OwnerDropped,
                HolderStage::OldOnly,
                HolderStage::Released
            ]
        );
    }
}
