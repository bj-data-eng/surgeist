//! One explicitly selected native population/sample per process. The optional
//! single-stage gate lends no DOM authority and never owns a semantic holder.
//! The runner owns its fresh directory and any external measurement process.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use super::{
    HolderStage, Population, construction_peak, require_release, run_sample, scale_metrics,
};

const SIZES: [usize; 5] = [1000, 10_000, 100_000, 800_000, 1_000_000];
const MAX_ACK_BYTES: usize = 512;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Sample {
    Warmup,
    First,
    Second,
    Third,
}

impl Sample {
    fn name(self) -> &'static str {
        match self {
            Self::Warmup => "warmup",
            Self::First => "0",
            Self::Second => "1",
            Self::Third => "2",
        }
    }

    fn ordinal(self) -> usize {
        match self {
            Self::Warmup | Self::First => 0,
            Self::Second => 1,
            Self::Third => 2,
        }
    }
}

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
        let sample = match sample {
            "warmup" => Sample::Warmup,
            "0" => Sample::First,
            "1" => Sample::Second,
            "2" => Sample::Third,
            _ => return Err("invalid SURGEIST_MEMORY_SAMPLE"),
        };
        Ok(Self {
            population,
            managed,
            sample,
        })
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
    directory: PathBuf,
    token: String,
    timeout: Duration,
}

impl GateConfig {
    fn parse(
        hold: &str,
        directory: Option<&str>,
        token: Option<&str>,
        timeout: Option<&str>,
    ) -> Result<Option<Self>, &'static str> {
        if hold == "none" {
            if directory.is_some() || token.is_some() || timeout.is_some() {
                return Err("hold=none forbids inherited acknowledgement configuration");
            }
            return Ok(None);
        }
        let stage = stage(hold)?;
        let directory = directory
            .filter(|value| !value.is_empty())
            .ok_or("missing SURGEIST_MEMORY_STAGE_DIR")?;
        let token = token.ok_or("missing SURGEIST_MEMORY_TOKEN")?;
        if token.is_empty()
            || token.len() > 64
            || !token
                .bytes()
                .all(|value| value.is_ascii_alphanumeric() || value == b'-' || value == b'_')
        {
            return Err(
                "SURGEIST_MEMORY_TOKEN requires 1..64 ASCII letters/digits/hyphens/underscores",
            );
        }
        let seconds: u64 = timeout
            .ok_or("missing SURGEIST_MEMORY_ACK_TIMEOUT_SECONDS")?
            .parse()
            .map_err(|_| "invalid acknowledgement timeout")?;
        if !(1..=300).contains(&seconds) {
            return Err("acknowledgement timeout must be 1..300 seconds");
        }
        Ok(Some(Self {
            stage,
            directory: directory.into(),
            token: token.into(),
            timeout: Duration::from_secs(seconds),
        }))
    }
}

fn required(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("explicit UTF-8 {name} is required"))
}

fn optional(name: &str) -> Option<String> {
    match std::env::var(name) {
        Ok(value) => Some(value),
        Err(std::env::VarError::NotPresent) => None,
        Err(std::env::VarError::NotUnicode(_)) => panic!("{name} must be UTF-8"),
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

fn empty_directory(directory: &Path) -> io::Result<()> {
    if let Some(entry) = fs::read_dir(directory)?.next() {
        entry?;
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "runner stage directory must be fresh and empty",
        ));
    }
    Ok(())
}

struct Gate {
    config: GateConfig,
    completed: bool,
}

impl Gate {
    fn open(config: GateConfig) -> io::Result<Self> {
        empty_directory(&config.directory)?;
        Ok(Self {
            config,
            completed: false,
        })
    }

    fn checkpoint(&mut self, selected: Selection, pid: u32, at: HolderStage) -> io::Result<()> {
        if at != self.config.stage {
            return Ok(());
        }
        assert!(!self.completed, "the one selected gate occurs once");
        let pending = PendingAck::publish(&self.config, selected, pid)?;
        pending.wait()?;
        self.completed = true;
        Ok(())
    }
}

struct PendingAck {
    path: PathBuf,
    expected: String,
    timeout: Duration,
}

impl PendingAck {
    fn publish(config: &GateConfig, selected: Selection, pid: u32) -> io::Result<Self> {
        // Check again at readiness, rejecting premature/stale acknowledgements.
        // The runner writes only ack.pending -> ack after reading ready.
        empty_directory(&config.directory)?;
        let expected = format!(
            "token={}\npid={pid}\npopulation={}\nmanaged={}\nsample={}\nstage={}\n",
            config.token,
            selected.population.name(),
            selected.managed,
            selected.sample.name(),
            config.stage.name()
        );
        assert!(expected.len() <= MAX_ACK_BYTES);
        let temporary = config.directory.join("ready.pending");
        let ready = config.directory.join("ready");
        {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)?;
            file.write_all(expected.as_bytes())?;
            file.flush()?;
        }
        // Same-directory rename publishes the complete, closed record. The
        // fresh directory has exactly one ready writer: this fixture.
        fs::rename(temporary, ready)?;
        Ok(Self {
            path: config.directory.join("ack"),
            expected,
            timeout: config.timeout,
        })
    }

    fn acknowledged(&self) -> io::Result<bool> {
        match fs::symlink_metadata(&self.path) {
            Ok(metadata) if metadata.file_type().is_file() => {}
            Ok(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "acknowledgement must be a regular file",
                ));
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
            Err(error) => return Err(error),
        }
        let file = match File::open(&self.path) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
            Err(error) => return Err(error),
        };
        let mut bytes = Vec::new();
        file.take((MAX_ACK_BYTES + 1) as u64)
            .read_to_end(&mut bytes)?;
        if bytes != self.expected.as_bytes() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "acknowledgement must match the complete token/PID/selection/stage record exactly",
            ));
        }
        Ok(true)
    }

    fn wait(&self) -> io::Result<()> {
        let started = Instant::now();
        loop {
            let acknowledged = self.acknowledged()?;
            let elapsed = started.elapsed();
            if elapsed >= self.timeout {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    format!(
                        "stage acknowledgement missing after {elapsed:?}; expected {}",
                        self.expected
                    ),
                ));
            }
            if acknowledged {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(10).min(self.timeout - elapsed));
        }
    }
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
        .map(Gate::open)
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
            gate.checkpoint(selected, pid, at)
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
            gate.completed,
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
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct Directory(PathBuf);
    impl Directory {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let next = NEXT
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                    value.checked_add(1)
                })
                .unwrap();
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "surgeist-memory-selected-{}-{now}-{next}",
                std::process::id()
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Directory {
        fn drop(&mut self) {
            if let Err(error) = fs::remove_dir_all(&self.0) {
                if std::thread::panicking() {
                    eprintln!("owned stage-test directory cleanup failed: {error}");
                } else {
                    panic!("owned stage-test directory cleanup failed: {error}");
                }
            }
        }
    }

    fn config(directory: &Directory) -> GateConfig {
        GateConfig::parse(
            "edited",
            Some(directory.0.to_str().unwrap()),
            Some("unit-token"),
            Some("1"),
        )
        .unwrap()
        .unwrap()
    }
    fn selected() -> Selection {
        Selection::parse("detached-text", "1000", "0").unwrap()
    }

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
    fn file_gate_accepts_only_the_exact_ready_record_and_keeps_runner_artifacts() {
        let directory = Directory::new();
        let configuration = config(&directory);
        let pending = PendingAck::publish(&configuration, selected(), 42).unwrap();
        let expected = b"token=unit-token\npid=42\npopulation=detached-text\nmanaged=1000\nsample=0\nstage=edited\n";
        assert_eq!(fs::read(directory.0.join("ready")).unwrap(), expected);
        assert!(!pending.acknowledged().unwrap());
        fs::write(directory.0.join("ack.pending"), expected).unwrap();
        fs::rename(directory.0.join("ack.pending"), directory.0.join("ack")).unwrap();
        pending.wait().unwrap();
        assert!(directory.0.join("ready").is_file());
        assert!(directory.0.join("ack").is_file());
    }

    #[test]
    fn file_gate_refuses_stale_and_malformed_acknowledgements() {
        let directory = Directory::new();
        fs::write(directory.0.join("ack"), "stale").unwrap();
        assert_eq!(
            Gate::open(config(&directory)).err().unwrap().kind(),
            io::ErrorKind::AlreadyExists
        );
        for (old, new) in [
            ("unit-token", "foreign"),
            ("pid=42", "pid=43"),
            ("detached-text", "attached-text"),
            ("managed=1000", "managed=10000"),
            ("sample=0", "sample=1"),
            ("stage=edited", "stage=released"),
        ] {
            let directory = Directory::new();
            let pending = PendingAck::publish(&config(&directory), selected(), 42).unwrap();
            fs::write(directory.0.join("ack"), pending.expected.replace(old, new)).unwrap();
            assert_eq!(
                pending.acknowledged().unwrap_err().kind(),
                io::ErrorKind::InvalidData
            );
        }
        let directory = Directory::new();
        let pending = PendingAck::publish(&config(&directory), selected(), 42).unwrap();
        fs::write(directory.0.join("ack"), vec![b'x'; MAX_ACK_BYTES + 1]).unwrap();
        assert_eq!(
            pending.acknowledged().unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        let directory = Directory::new();
        let pending = PendingAck::publish(&config(&directory), selected(), 42).unwrap();
        fs::create_dir(directory.0.join("ack")).unwrap();
        assert_eq!(
            pending.acknowledged().unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }

    #[test]
    fn missing_acknowledgement_at_expired_deadline_refuses_without_waiting() {
        let directory = Directory::new();
        let mut pending = PendingAck::publish(&config(&directory), selected(), 42).unwrap();
        // Direct expired-deadline condition; no arbitrary wall-clock assertion.
        pending.timeout = Duration::ZERO;
        assert_eq!(pending.wait().unwrap_err().kind(), io::ErrorKind::TimedOut);
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
