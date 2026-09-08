//! Bounded synchronization for private tests of the proxy wake protocol.
//!
//! The wake double deliberately delays its return to expose concurrent proxy
//! transitions. It never calls back into the proxy or waits for a drain.

use std::{
    fmt::Display,
    fs::{self, File},
    io::Read,
    path::PathBuf,
    process::{Child, Command, ExitStatus, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
        mpsc::{self, Receiver, Sender},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use crate::{WakeBridge, WakeError};

const CHILD_SCENARIO: &str = "SURGEIST_RUNTIME_TEST_CHILD_SCENARIO";
const OUTPUT_LIMIT: u64 = 64 * 1024;
static NEXT_CAPTURE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone)]
pub(crate) struct ScenarioDeadline {
    started: Instant,
    timeout: Duration,
    events: Arc<Mutex<Vec<String>>>,
}

impl ScenarioDeadline {
    pub(crate) fn new() -> Self {
        // A single generous budget covers synchronization under the parallel suite.
        Self::with_timeout(Duration::from_secs(10))
    }

    pub(crate) fn with_timeout(timeout: Duration) -> Self {
        Self {
            started: Instant::now(),
            timeout,
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub(crate) fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }

    pub(crate) fn record(&self, event: impl Into<String>) {
        self.events.lock().unwrap().push(event.into());
    }

    pub(crate) fn diagnostic(&self, phase: &str, last_state: impl Display) -> String {
        format!(
            "phase={phase}; elapsed={:?}; deadline={:?}; last state={last_state}; events={:?}",
            self.elapsed(),
            self.timeout,
            self.events.lock().unwrap(),
        )
    }

    pub(crate) fn remaining(&self, phase: &str) -> Result<Duration, String> {
        self.timeout
            .checked_sub(self.elapsed())
            .filter(|remaining| !remaining.is_zero())
            .ok_or_else(|| self.diagnostic(phase, "deadline expired"))
    }

    pub(crate) fn receive<T>(&self, phase: &str, receiver: &Receiver<T>) -> Result<T, String> {
        let remaining = self.remaining(phase)?;
        receiver.recv_timeout(remaining).map_err(|error| {
            let state = match error {
                mpsc::RecvTimeoutError::Timeout => "completion channel timed out",
                mpsc::RecvTimeoutError::Disconnected => "completion channel disconnected",
            };
            self.diagnostic(phase, state)
        })
    }

    pub(crate) fn wait_for(
        &self,
        phase: &str,
        expected: usize,
        mut observe: impl FnMut() -> usize,
    ) -> Result<(), String> {
        loop {
            let last_observed = observe();
            if last_observed == expected {
                self.record(format!("{phase}: observed {expected}"));
                return Ok(());
            }
            if self.remaining(phase).is_err() {
                return Err(self.diagnostic(
                    phase,
                    format_args!("expected {expected}, last observed {last_observed}"),
                ));
            }
            // These private queue milestones expose state, not completion signals.
            thread::yield_now();
        }
    }

    pub(crate) fn join<T>(&self, phase: &str, worker: JoinHandle<T>) -> Result<T, String> {
        self.wait_for(phase, 1, || usize::from(worker.is_finished()))?;
        worker
            .join()
            .map_err(|_| self.diagnostic(phase, "worker panicked"))
    }
}

pub(crate) struct BlockingWakeBridge {
    deadline: ScenarioDeadline,
    wakes: Mutex<usize>,
    started: Sender<WakeRelease>,
}

pub(crate) struct WakeController {
    deadline: ScenarioDeadline,
    started: Receiver<WakeRelease>,
}

pub(crate) struct WakeRelease {
    ordinal: usize,
    deadline: ScenarioDeadline,
    release: Sender<Result<(), WakeError>>,
}

impl BlockingWakeBridge {
    pub(crate) fn new(deadline: &ScenarioDeadline) -> (Self, WakeController) {
        let (started_tx, started_rx) = mpsc::channel();
        (
            Self {
                deadline: deadline.clone(),
                wakes: Mutex::new(0),
                started: started_tx,
            },
            WakeController {
                deadline: deadline.clone(),
                started: started_rx,
            },
        )
    }
}

impl WakeController {
    pub(crate) fn next(&self, expected: usize) -> Result<WakeRelease, String> {
        let phase = format!("wake {expected} start");
        let gate = self.deadline.receive(&phase, &self.started)?;
        if gate.ordinal != expected {
            return Err(self.deadline.diagnostic(
                &phase,
                format_args!("expected wake {expected}, observed wake {}", gate.ordinal),
            ));
        }
        Ok(gate)
    }
}

impl WakeRelease {
    pub(crate) fn release(self, outcome: Result<(), WakeError>) -> Result<(), String> {
        self.deadline
            .record(format!("wake {} released: {outcome:?}", self.ordinal));
        self.release.send(outcome).map_err(|_| {
            self.deadline.diagnostic(
                "wake release",
                format_args!("wake {} receiver disconnected", self.ordinal),
            )
        })
    }
}

impl WakeBridge for BlockingWakeBridge {
    fn wake(&self) -> Result<(), WakeError> {
        let ordinal = {
            let mut wakes = self.wakes.lock().unwrap();
            *wakes += 1;
            *wakes
        };
        let phase = format!("wake {ordinal} release");
        let (release, receiver) = mpsc::channel();
        self.deadline.record(format!("wake {ordinal} started"));
        self.started
            .send(WakeRelease {
                ordinal,
                deadline: self.deadline.clone(),
                release,
            })
            .map_err(|_| {
                WakeError::new(
                    self.deadline
                        .diagnostic(&phase, "wake controller disconnected"),
                )
            })?;
        self.deadline
            .receive(&phase, &receiver)
            .map_err(WakeError::new)?
    }
}

pub(crate) fn run_in_child(name: &str, scenario: impl FnOnce()) {
    if is_scenario_child(name) {
        println!("SCENARIO_STARTED:{name}");
        scenario();
        println!("SCENARIO_COMPLETED:{name}");
        return;
    }

    let mut child = ScenarioChild::spawn(name).unwrap_or_else(|error| panic!("{error}"));
    child
        .wait(Duration::from_secs(15))
        .unwrap_or_else(|error| panic!("{error}"));
}

fn is_scenario_child(name: &str) -> bool {
    std::env::var(CHILD_SCENARIO).is_ok_and(|selected| selected == name)
}

struct ScenarioChild {
    child: Option<Child>,
    directory: PathBuf,
    name: String,
}

impl ScenarioChild {
    fn spawn(name: &str) -> Result<Self, String> {
        let directory = loop {
            let sequence = NEXT_CAPTURE.fetch_add(1, Ordering::Relaxed);
            let directory = std::env::temp_dir().join(format!(
                "surgeist-runtime-scenario-{}-{sequence}",
                std::process::id()
            ));
            match fs::create_dir(&directory) {
                Ok(()) => break directory,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(format!("create scenario output directory: {error}")),
            }
        };
        let mut guard = Self {
            child: None,
            directory,
            name: name.to_owned(),
        };
        let stdout = File::create(guard.directory.join("stdout"))
            .map_err(|error| format!("create scenario stdout: {error}"))?;
        let stderr = File::create(guard.directory.join("stderr"))
            .map_err(|error| format!("create scenario stderr: {error}"))?;
        guard.child = Some(
            Command::new(std::env::current_exe().map_err(|error| error.to_string())?)
                .args(["--exact", name, "--nocapture", "--test-threads=1"])
                .env(CHILD_SCENARIO, name)
                .stdin(Stdio::null())
                .stdout(stdout)
                .stderr(stderr)
                .spawn()
                .map_err(|error| format!("spawn scenario {name}: {error}"))?,
        );
        Ok(guard)
    }

    fn output(&self) -> String {
        ["stdout", "stderr"]
            .into_iter()
            .map(|stream| {
                let mut bytes = Vec::new();
                match File::open(self.directory.join(stream))
                    .and_then(|file| file.take(OUTPUT_LIMIT).read_to_end(&mut bytes))
                {
                    Ok(_) => format!(
                        "{stream} (first {OUTPUT_LIMIT} bytes):\n{}",
                        String::from_utf8_lossy(&bytes)
                    ),
                    Err(error) => format!("{stream}: capture read failed: {error}"),
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn wait(&mut self, timeout: Duration) -> Result<(), String> {
        let deadline = ScenarioDeadline::with_timeout(timeout);
        let result = loop {
            match self.child.as_mut().unwrap().try_wait() {
                Ok(Some(status)) => break Ok(status),
                Ok(None) => {}
                Err(error) => break Err(deadline.diagnostic("child status", error)),
            }
            let remaining = match deadline.remaining("child completion") {
                Ok(remaining) => remaining,
                Err(error) => break Err(error),
            };
            // std::process offers no exit subscription; poll fresh status within the deadline.
            thread::sleep(remaining.min(Duration::from_millis(5)));
        };
        let output = self.output();
        let cleanup = self.cleanup();
        cleanup?;
        let status = result.map_err(|error| format!("{}: {error}\n{output}", self.name))?;
        if !status.success()
            || !output.contains("running 1 test")
            || !output.contains(&format!("SCENARIO_STARTED:{}", self.name))
            || !output.contains(&format!("SCENARIO_COMPLETED:{}", self.name))
        {
            return Err(format!(
                "scenario {} did not complete its exact test; status={status}; elapsed={:?}\n{output}",
                self.name,
                deadline.elapsed(),
            ));
        }
        Ok(())
    }

    fn cleanup(&mut self) -> Result<Option<ExitStatus>, String> {
        let process_result = self.cleanup_process();
        let output_result = match fs::remove_dir_all(&self.directory) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(format!("remove owned scenario output: {error}")),
        };
        match (process_result, output_result) {
            (Ok(status), Ok(())) => Ok(status),
            (Err(error), Ok(())) | (Ok(_), Err(error)) => Err(error),
            (Err(process_error), Err(output_error)) => {
                Err(format!("{process_error}; {output_error}"))
            }
        }
    }

    fn cleanup_process(&mut self) -> Result<Option<ExitStatus>, String> {
        let status = if let Some(child) = self.child.as_mut() {
            let observed = child
                .try_wait()
                .map_err(|error| format!("inspect owned child during cleanup: {error}"))?;
            let status = if let Some(status) = observed {
                status
            } else {
                child
                    .kill()
                    .map_err(|error| format!("kill owned scenario child: {error}"))?;
                // Killing is immediate for these children; allow scheduler latency
                // while keeping cleanup independently bounded after a scenario timeout.
                let deadline = ScenarioDeadline::with_timeout(Duration::from_secs(1));
                loop {
                    if let Some(status) = child
                        .try_wait()
                        .map_err(|error| format!("reap owned scenario child: {error}"))?
                    {
                        break status;
                    }
                    let remaining = deadline.remaining("reap killed scenario child")?;
                    thread::sleep(remaining.min(Duration::from_millis(5)));
                }
            };
            // A successful try_wait reaps the process; no unbounded wait is needed.
            Some(status)
        } else {
            None
        };
        self.child = None;
        Ok(status)
    }
}

impl Drop for ScenarioChild {
    fn drop(&mut self) {
        if let Err(error) = self.cleanup() {
            eprintln!("scenario cleanup failed: {error}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expired_deadline_reports_phase_and_recorded_events() {
        let deadline = ScenarioDeadline::with_timeout(Duration::ZERO);
        deadline.record("sender queued");
        let (_wake, controller) = BlockingWakeBridge::new(&deadline);
        let error = controller.next(3).err().expect("deadline must expire");
        assert!(error.contains("phase=wake 3 start"));
        assert!(error.contains("deadline expired"));
        assert!(error.contains("sender queued"));
        assert!(error.contains("elapsed="));
    }

    #[test]
    fn disconnected_wake_release_returns_wake_error() {
        let deadline = ScenarioDeadline::new();
        let (wake, controller) = BlockingWakeBridge::new(&deadline);
        let worker = thread::spawn(move || wake.wake());
        drop(controller.next(1).unwrap());
        let error = deadline
            .join("wake completion", worker)
            .unwrap()
            .unwrap_err();
        assert!(error.to_string().contains("wake 1 release"));
        assert!(error.to_string().contains("disconnected"));
        assert!(error.to_string().contains("wake 1 started"));
    }

    #[test]
    fn expired_wake_release_returns_wake_error() {
        let deadline = ScenarioDeadline::with_timeout(Duration::ZERO);
        let (wake, _controller) = BlockingWakeBridge::new(&deadline);
        let error = wake.wake().unwrap_err();
        assert!(error.to_string().contains("wake 1 release"));
        assert!(error.to_string().contains("deadline expired"));
    }

    #[test]
    fn child_timeout_reaps_process_and_removes_output() {
        let name = "testing::concurrency::tests::child_timeout_reaps_process_and_removes_output";
        if is_scenario_child(name) {
            loop {
                thread::park();
            }
        }
        let mut child = ScenarioChild::spawn(name).unwrap();
        let directory = child.directory.clone();
        let error = child.wait(Duration::ZERO).unwrap_err();
        assert!(error.contains("child completion"));
        assert!(error.contains("deadline expired"));
        assert!(
            !directory.exists(),
            "owned output directory must be removed"
        );
    }

    #[test]
    fn child_cleanup_terminates_and_reaps_live_process() {
        let name = "testing::concurrency::tests::child_cleanup_terminates_and_reaps_live_process";
        if is_scenario_child(name) {
            loop {
                thread::park();
            }
        }
        let mut child = ScenarioChild::spawn(name).unwrap();
        let directory = child.directory.clone();
        assert!(child.child.as_mut().unwrap().try_wait().unwrap().is_none());
        let status = child.cleanup().unwrap().unwrap();
        assert!(!status.success(), "cleanup must terminate the live child");
        drop(child);
        assert!(
            !directory.exists(),
            "owned output directory must be removed"
        );
    }

    #[test]
    fn child_guard_removes_output_during_panic() {
        let name = "testing::concurrency::tests::child_guard_removes_output_during_panic";
        if is_scenario_child(name) {
            loop {
                thread::park();
            }
        }
        let mut child = ScenarioChild::spawn(name).unwrap();
        let directory = child.directory.clone();
        assert!(child.child.as_mut().unwrap().try_wait().unwrap().is_none());
        let result = std::panic::catch_unwind(move || {
            let _guard = child;
            panic!("intentional scenario failure");
        });
        assert!(result.is_err());
        assert!(
            !directory.exists(),
            "unwinding must clean the owned scenario output directory"
        );
    }
}
