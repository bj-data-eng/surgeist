#![forbid(unsafe_code)]
#![cfg(all(feature = "execution", target_os = "macos", target_arch = "aarch64"))]

use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::Duration,
};
use surgeist_fuzz::{
    engine_minimization::{EngineMinimizer, ProposalLimits},
    execution::{CleanupStatus, MemoryPolicy, ProcessExit, ProcessStop, WorkerLimits},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Root(PathBuf);
impl Root {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "surgeist-proposal-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path.canonicalize().unwrap())
    }
    fn controller(&self, executable: PathBuf) -> EngineMinimizer {
        EngineMinimizer::new(
            executable,
            self.0.clone(),
            WorkerLimits::new(
                0,
                2 * 1024 * 1024,
                2 * 1024 * 1024,
                Duration::from_secs(10),
                Duration::from_millis(100),
                Duration::from_secs(3),
                MemoryPolicy::Unbounded,
            )
            .unwrap(),
            ProposalLimits::new(100, 16, 128, 2048).unwrap(),
        )
        .unwrap()
    }
}
impl Drop for Root {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn invalid_limits_and_oversized_originals_never_start_engine_work() {
    for values in [(0, 1, 1, 1), (1, 0, 1, 1), (1, 1, 0, 1), (1, 1, 2, 1)] {
        assert!(ProposalLimits::new(values.0, values.1, values.2, values.3).is_err());
    }
    let root = Root::new();
    let controller = root.controller(PathBuf::from(env!(
        "CARGO_BIN_EXE_surgeist-fuzz-process-worker"
    )));
    assert!(
        controller
            .failure_proposal(&[0; 129], &AtomicBool::new(false))
            .is_err()
    );
    assert_eq!(fs::read_dir(&root.0).unwrap().count(), 0);
}

#[test]
fn unsuccessful_engine_retains_original_and_reports_its_actual_exit() {
    let root = Root::new();
    let controller = root.controller(PathBuf::from(env!(
        "CARGO_BIN_EXE_surgeist-fuzz-process-worker"
    )));
    let original = b"exact original input";
    let result = controller
        .failure_proposal(original, &AtomicBool::new(false))
        .unwrap();
    assert_eq!(result.originals()[0].bytes(), original);
    assert!(result.proposed().is_empty());
    assert_eq!(result.process().stop, ProcessStop::Exited);
    assert_eq!(result.process().exit, ProcessExit::Nonzero(19));
    assert_eq!(result.process().cleanup, CleanupStatus::Complete);
    assert_eq!(
        fs::read(result.directory().join("original.entropy")).unwrap(),
        original
    );
}

#[test]
fn source_corpus_symlinks_and_quotas_are_rejected_without_modifying_inputs() {
    let root = Root::new();
    let controller = root.controller(PathBuf::from(env!(
        "CARGO_BIN_EXE_surgeist-fuzz-process-worker"
    )));
    let source = root.0.join("source");
    fs::create_dir(&source).unwrap();
    fs::write(source.join("large"), [0; 129]).unwrap();
    assert!(
        controller
            .coverage_corpus_proposal(&source, &AtomicBool::new(false))
            .is_err()
    );
    assert_eq!(fs::read(source.join("large")).unwrap(), [0; 129]);
    fs::remove_file(source.join("large")).unwrap();
    fs::write(root.0.join("outside"), b"outside").unwrap();
    std::os::unix::fs::symlink(root.0.join("outside"), source.join("linked")).unwrap();
    assert!(
        controller
            .coverage_corpus_proposal(&source, &AtomicBool::new(false))
            .is_err()
    );
    assert_eq!(fs::read(root.0.join("outside")).unwrap(), b"outside");
}

#[test]
#[ignore = "requires an explicitly supplied prepared engine executable"]
fn prepared_engine_proposes_smaller_input_while_preserving_original_bytes() {
    let root = Root::new();
    let controller = root.controller(PathBuf::from(
        std::env::var_os("SURGEIST_FUZZ_FAILURE_ENGINE").expect("prepared executable"),
    ));
    let original = b"lots of padding before BUG! and after";
    let result = controller
        .failure_proposal(original, &AtomicBool::new(false))
        .unwrap();
    assert_eq!(result.process().cleanup, CleanupStatus::Complete);
    assert_eq!(result.process().stop, ProcessStop::Exited);
    assert!(result.readback_error().is_none(), "{result:?}");
    assert!(!result.proposed().is_empty(), "{result:?}");
    assert!(result.proposed()[0].bytes().len() < original.len());
    assert_eq!(result.originals()[0].bytes(), original);
    assert_eq!(
        fs::read(result.directory().join("original.entropy")).unwrap(),
        original
    );
}

#[test]
#[ignore = "requires an explicitly supplied prepared engine executable"]
fn prepared_engine_reduces_coverage_corpus_without_changing_checked_in_inputs() {
    let root = Root::new();
    let controller = root.controller(PathBuf::from(
        std::env::var_os("SURGEIST_FUZZ_CORPUS_ENGINE").expect("prepared executable"),
    ));
    let source = root.0.join("source");
    fs::create_dir(&source).unwrap();
    let originals = [vec![0], vec![1], vec![2], vec![0, 0], vec![1, 1]];
    for (index, input) in originals.iter().enumerate() {
        fs::write(source.join(index.to_string()), input).unwrap();
    }
    let result = controller
        .coverage_corpus_proposal(&source, &AtomicBool::new(false))
        .unwrap();
    assert_eq!(result.process().cleanup, CleanupStatus::Complete);
    assert_eq!(result.process().exit, ProcessExit::Success, "{result:?}");
    assert!(result.readback_error().is_none(), "{result:?}");
    assert!(!result.proposed().is_empty());
    assert!(result.proposed().len() <= originals.len());
    for (index, input) in originals.iter().enumerate() {
        assert_eq!(&fs::read(source.join(index.to_string())).unwrap(), input);
    }
}
