//! Owned mutable storage for finite rounds of a logical fuzz campaign.
#![forbid(unsafe_code)]

use crate::{FuzzError, FuzzErrorKind, Result, Sha256Digest};
use rustix::fd::OwnedFd;
use rustix::fs::{self, AtFlags, Dir, FileType, Mode, OFlags};
use serde::Serialize;
use std::{
    collections::BTreeMap,
    fs::File,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

// Directories, committed control files and their simultaneous replacement files.
const FIXED_ENTRIES: u64 = 16;
const ROOT_NAMES: &[&str] = &[
    "entropy-corpus",
    "engine-artifacts",
    "retained-a",
    "retained-b",
    "progress.json",
    "progress.json.partial",
    "corpus-basis.json",
    "corpus-basis.json.partial",
    "engine.stderr",
    "engine.stderr.partial",
    "corpus-commit",
    "corpus-commit.partial",
];

fn invalid(detail: impl Into<String>) -> FuzzError {
    FuzzError::new(FuzzErrorKind::InvalidLimits, "fuzz session storage", detail)
}
fn resource(detail: impl Into<String>) -> FuzzError {
    FuzzError::new(FuzzErrorKind::Resource, "fuzz session storage", detail)
}
fn io_error(error: impl std::error::Error + Send + Sync + 'static) -> FuzzError {
    FuzzError::with_source(
        FuzzErrorKind::Io,
        "fuzz session storage",
        error.to_string(),
        error,
    )
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct SessionLimits {
    total_entries: u64,
    total_bytes: u64,
    input_bytes: u64,
    round_cases: u64,
    retained_entries: u64,
    log_bytes: u64,
    checkpoint_bytes: u64,
    reserved_entries: u64,
    reserved_bytes: u64,
}

impl SessionLimits {
    /// Requested caps are reduced explicitly to fit the simultaneous envelope.
    /// Startup retains room for its empty callback and at least one mutation.
    pub(crate) fn new(
        total_entries: u64,
        total_bytes: u64,
        input_bytes: u64,
        round_cases: u64,
        retained_entries: u64,
        log_bytes: u64,
        checkpoint_bytes: u64,
    ) -> Result<Self> {
        if total_entries == 0
            || total_entries > 100_000
            || total_bytes == 0
            || input_bytes == 0
            || input_bytes > 64 * 1024 * 1024
            || round_cases < 3
            || retained_entries == 0
            || log_bytes == 0
            || checkpoint_bytes == 0
        {
            return Err(invalid(
                "positive finite session limits and at least three round cases required",
            ));
        }
        let controls = log_bytes
            .checked_mul(2)
            .and_then(|n| {
                checkpoint_bytes
                    .checked_mul(4)
                    .and_then(|m| n.checked_add(m))
            })
            .and_then(|n| n.checked_add(2))
            .ok_or_else(|| invalid("control byte envelope overflow"))?;
        let slots = total_entries
            .checked_sub(FIXED_ENTRIES)
            .zip(total_bytes.checked_sub(controls))
            .map(|(entries, bytes)| entries.min(bytes / input_bytes))
            .ok_or_else(|| invalid("session quota cannot admit control replacements"))?;
        // Active inputs: round_cases + retained + four startup/artifact allowances.
        // Both retained generations coexist during atomic rotation.
        let retained_entries = retained_entries
            .min(round_cases - 2)
            .min(slots.saturating_sub(6) / 4);
        if retained_entries == 0 {
            return Err(invalid(
                "session quota cannot admit one retained input and round",
            ));
        }
        let round_cases = round_cases.min(slots - 3 * retained_entries - 4);
        let input_slots = round_cases + 3 * retained_entries + 4;
        let reserved_bytes = input_slots
            .checked_mul(input_bytes)
            .and_then(|n| n.checked_add(controls))
            .ok_or_else(|| invalid("session byte envelope overflow"))?;
        Ok(Self {
            total_entries,
            total_bytes,
            input_bytes,
            round_cases,
            retained_entries,
            log_bytes,
            checkpoint_bytes,
            reserved_entries: input_slots + FIXED_ENTRIES,
            reserved_bytes,
        })
    }
    pub(crate) const fn round_cases(self) -> u64 {
        self.round_cases
    }
    pub(crate) const fn retained_entries(self) -> u64 {
        self.retained_entries
    }
    fn active_entries(self) -> u64 {
        self.round_cases + self.retained_entries + 4
    }
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct RetentionReport {
    pub retained_entries: u64,
    pub retained_bytes: u64,
    pub discarded_entries: u64,
    pub discarded_bytes: u64,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct SessionReport {
    pub round_cases: u64,
    pub retained_limit: u64,
    pub retained_entries: u64,
    pub retained_bytes: u64,
    pub active_entries: u64,
    pub active_bytes: u64,
    pub engine_artifact_entries: u64,
    pub engine_artifact_bytes: u64,
    pub reserved_entries: u64,
    pub reserved_bytes: u64,
    pub partial_replacements: Vec<String>,
}

pub(crate) struct SessionStore {
    root: OwnedFd,
    limits: SessionLimits,
    corpus_path: PathBuf,
    artifacts_path: PathBuf,
}

impl SessionStore {
    /// Creates a new owned directory; existing roots are never adopted or swept.
    pub(crate) fn create(root: impl AsRef<Path>, limits: SessionLimits) -> Result<Self> {
        let path = if root.as_ref().is_absolute() {
            root.as_ref().to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(io_error)?
                .join(root.as_ref())
        };
        let parent = open_root(
            path.parent()
                .ok_or_else(|| invalid("session root has no parent"))?,
        )?;
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| invalid("invalid session root name"))?;
        fs::mkdirat(&parent, name, Mode::from_raw_mode(0o700)).map_err(io_error)?;
        let root = directory(&parent, name)?;
        for name in [
            "entropy-corpus",
            "engine-artifacts",
            "retained-a",
            "retained-b",
        ] {
            fs::mkdirat(&root, name, Mode::from_raw_mode(0o700)).map_err(io_error)?;
        }
        write_new(&root, "corpus-commit", b"a")?;
        fs::fsync(&root).map_err(io_error)?;
        fs::fsync(&parent).map_err(io_error)?;
        Ok(Self {
            root,
            limits,
            corpus_path: path.join("entropy-corpus"),
            artifacts_path: path.join("engine-artifacts"),
        })
    }
    pub(crate) fn corpus_path(&self) -> &Path {
        &self.corpus_path
    }
    pub(crate) fn retained_corpus_path(&self) -> Result<PathBuf> {
        Ok(self
            .corpus_path
            .parent()
            .expect("owned corpus has a parent")
            .join(self.committed_name()?))
    }
    pub(crate) fn engine_artifacts_path(&self) -> &Path {
        &self.artifacts_path
    }

    /// Campaign validates seed provenance. Input loss is reported explicitly.
    pub(crate) fn initialize(&mut self, inputs: &[Vec<u8>]) -> Result<RetentionReport> {
        self.report()?;
        if inputs.is_empty() || inputs.len() as u64 > self.limits.total_entries {
            return Err(resource(
                "seed count exceeds initialization allowance or is empty",
            ));
        }
        let bytes = inputs.iter().try_fold(0_u64, |total, bytes| {
            total
                .checked_add(bytes.len() as u64)
                .ok_or_else(|| resource("seed byte count overflow"))
        })?;
        if bytes > self.limits.total_bytes {
            return Err(resource("seed bytes exceed initialization allowance"));
        }
        let candidates = inputs
            .iter()
            .map(|bytes| self.candidate(bytes.clone()))
            .collect::<Result<Vec<_>>>()?;
        let report = self.commit_candidates(candidates)?;
        self.reset_round()?;
        Ok(report)
    }

    /// Caller must first establish complete cleanup of the engine process group.
    /// Selection is lossy small-first retention, not a coverage-preservation claim.
    pub(crate) fn finish_round(&mut self) -> Result<RetentionReport> {
        self.report()?;
        let mut candidates = self.read_inputs("entropy-corpus", self.limits.active_entries())?;
        candidates.extend(self.read_inputs(self.committed_name()?, self.limits.retained_entries)?);
        self.commit_candidates(candidates)
    }

    /// Only called while the engine is stopped. Committed retention survives errors.
    pub(crate) fn reset_round(&mut self) -> Result<()> {
        self.report()?;
        let inputs = self.read_inputs(self.committed_name()?, self.limits.retained_entries)?;
        let corpus = directory(&self.root, "entropy-corpus")?;
        clear_directory(&corpus, self.limits.active_entries())?;
        let artifacts = directory(&self.root, "engine-artifacts")?;
        clear_directory(&artifacts, self.limits.active_entries())?;
        for candidate in inputs {
            write_new(&corpus, candidate.digest.as_str(), &candidate.bytes)?;
        }
        fs::fsync(&corpus).map_err(io_error)
    }
    pub(crate) fn replace_checkpoint(&mut self, bytes: &[u8]) -> Result<()> {
        self.replace("progress.json", bytes, self.limits.checkpoint_bytes)
    }
    pub(crate) fn write_basis(&mut self, bytes: &[u8]) -> Result<()> {
        self.replace("corpus-basis.json", bytes, self.limits.checkpoint_bytes)
    }
    /// The caller supplies an already bounded diagnostic tail.
    pub(crate) fn replace_log(&mut self, bytes: &[u8]) -> Result<()> {
        self.replace("engine.stderr", bytes, self.limits.log_bytes)
    }

    pub(crate) fn report(&self) -> Result<SessionReport> {
        let names = names(&self.root, FIXED_ENTRIES)?;
        let mut partial_replacements = Vec::new();
        let mut control_bytes = 0_u64;
        for (name, kind) in &names {
            if !ROOT_NAMES.contains(&name.as_str()) {
                return Err(resource("unexpected entry in owned session root"));
            }
            if matches!(
                name.as_str(),
                "entropy-corpus" | "engine-artifacts" | "retained-a" | "retained-b"
            ) {
                if *kind != FileType::Directory {
                    return Err(resource("session directory changed type"));
                }
            } else {
                let ceiling = if name.starts_with("engine.stderr") {
                    self.limits.log_bytes
                } else if name.starts_with("corpus-commit") {
                    1
                } else {
                    self.limits.checkpoint_bytes
                };
                control_bytes = control_bytes
                    .checked_add(file_size(&self.root, name, ceiling)?)
                    .ok_or_else(|| resource("session accounting overflow"))?;
                if name.ends_with(".partial") {
                    partial_replacements.push(name.clone());
                }
            }
        }
        let a = self.input_usage("retained-a", self.limits.retained_entries)?;
        let b = self.input_usage("retained-b", self.limits.retained_entries)?;
        let active = self.input_usage("entropy-corpus", self.limits.active_entries())?;
        let artifacts = self.input_usage("engine-artifacts", self.limits.active_entries())?;
        if active.0 + artifacts.0 > self.limits.active_entries() {
            return Err(resource("combined active input allowance exceeded"));
        }
        let entries = names.len() as u64 + a.0 + b.0 + active.0 + artifacts.0;
        let bytes = control_bytes
            .checked_add(a.1)
            .and_then(|n| n.checked_add(b.1))
            .and_then(|n| n.checked_add(active.1))
            .and_then(|n| n.checked_add(artifacts.1))
            .ok_or_else(|| resource("session accounting overflow"))?;
        if entries > self.limits.total_entries || bytes > self.limits.total_bytes {
            return Err(resource("session total storage quota exceeded"));
        }
        let retained = if self.committed_name()? == "retained-a" {
            a
        } else {
            b
        };
        let uncommitted = if self.committed_name()? == "retained-a" {
            b
        } else {
            a
        };
        if uncommitted.0 != 0 {
            partial_replacements.push(if self.committed_name()? == "retained-a" {
                "retained-b".into()
            } else {
                "retained-a".into()
            });
        }
        Ok(SessionReport {
            round_cases: self.limits.round_cases,
            retained_limit: self.limits.retained_entries,
            retained_entries: retained.0,
            retained_bytes: retained.1,
            active_entries: active.0,
            active_bytes: active.1,
            engine_artifact_entries: artifacts.0,
            engine_artifact_bytes: artifacts.1,
            reserved_entries: self.limits.reserved_entries,
            reserved_bytes: self.limits.reserved_bytes,
            partial_replacements,
        })
    }
    fn committed_name(&self) -> Result<&'static str> {
        match read(&self.root, "corpus-commit", 1)?.as_slice() {
            b"a" => Ok("retained-a"),
            b"b" => Ok("retained-b"),
            _ => Err(resource("invalid committed corpus selector")),
        }
    }
    fn input_usage(&self, name: &str, cap: u64) -> Result<(u64, u64)> {
        let dir = directory(&self.root, name)?;
        let entries = names(&dir, cap)?;
        let mut bytes = 0;
        for (name, _) in &entries {
            bytes += file_size(&dir, name, self.limits.input_bytes)?;
        }
        Ok((entries.len() as u64, bytes))
    }
    fn read_inputs(&self, name: &str, cap: u64) -> Result<Vec<Candidate>> {
        let dir = directory(&self.root, name)?;
        names(&dir, cap)?
            .into_iter()
            .map(|(name, _)| self.candidate(read(&dir, &name, self.limits.input_bytes)?))
            .collect()
    }
    fn candidate(&self, bytes: Vec<u8>) -> Result<Candidate> {
        if bytes.len() as u64 > self.limits.input_bytes {
            return Err(resource("entropy input exceeds selected ceiling"));
        }
        Ok(Candidate {
            digest: Sha256Digest::from_bytes(&bytes),
            bytes,
        })
    }
    fn commit_candidates(&mut self, candidates: Vec<Candidate>) -> Result<RetentionReport> {
        let total_entries = candidates.len() as u64;
        let total_bytes = candidates.iter().map(|c| c.bytes.len() as u64).sum::<u64>();
        let mut unique = BTreeMap::new();
        for candidate in candidates {
            if let Some(prior) = unique.insert(candidate.digest.clone(), candidate.bytes.clone())
                && prior != candidate.bytes
            {
                return Err(resource("entropy digest collision"));
            }
        }
        let mut selected: Vec<_> = unique.into_iter().collect();
        selected.sort_by(|(da, a), (db, b)| a.len().cmp(&b.len()).then_with(|| da.cmp(db)));
        selected.truncate(self.limits.retained_entries as usize);
        let old = self.committed_name()?;
        let next = if old == "retained-a" {
            "retained-b"
        } else {
            "retained-a"
        };
        let dir = directory(&self.root, next)?;
        clear_directory(&dir, self.limits.retained_entries)?;
        for (digest, bytes) in &selected {
            write_new(&dir, digest.as_str(), bytes)?;
        }
        fs::fsync(&dir).map_err(io_error)?;
        self.replace(
            "corpus-commit",
            if next == "retained-a" { b"a" } else { b"b" },
            1,
        )?;
        clear_directory(&directory(&self.root, old)?, self.limits.retained_entries)?;
        let retained_entries = selected.len() as u64;
        let retained_bytes = selected
            .iter()
            .map(|(_, bytes)| bytes.len() as u64)
            .sum::<u64>();
        Ok(RetentionReport {
            retained_entries,
            retained_bytes,
            discarded_entries: total_entries - retained_entries,
            discarded_bytes: total_bytes - retained_bytes,
        })
    }
    fn replace(&mut self, name: &str, bytes: &[u8], ceiling: u64) -> Result<()> {
        if bytes.len() as u64 > ceiling {
            return Err(resource("replacement exceeds selected byte ceiling"));
        }
        self.report()?;
        let partial = format!("{name}.partial");
        write_new(&self.root, &partial, bytes)?;
        fs::renameat(&self.root, &partial, &self.root, name).map_err(io_error)?;
        fs::fsync(&self.root).map_err(io_error)
    }
}

struct Candidate {
    digest: Sha256Digest,
    bytes: Vec<u8>,
}

fn open_root(path: &Path) -> Result<OwnedFd> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().map_err(io_error)?.join(path)
    };
    let mut fd = fs::open(
        "/",
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(io_error)?;
    for component in absolute.components() {
        match component {
            Component::RootDir => (),
            Component::Normal(name) => {
                fd = fs::openat(
                    &fd,
                    name,
                    OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                    Mode::empty(),
                )
                .map_err(io_error)?;
            }
            _ => return Err(resource("dot and parent path components are unsupported")),
        }
    }
    Ok(fd)
}
fn directory(parent: &OwnedFd, name: &str) -> Result<OwnedFd> {
    fs::openat(
        parent,
        name,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(io_error)
}
fn names(parent: &OwnedFd, ceiling: u64) -> Result<Vec<(String, FileType)>> {
    let mut dir = Dir::read_from(parent).map_err(io_error)?;
    let mut names = Vec::new();
    while let Some(entry) = dir.read() {
        let entry = entry.map_err(io_error)?;
        let name = entry
            .file_name()
            .to_str()
            .map_err(|_| resource("non-UTF-8 session entry"))?;
        if name == "." || name == ".." {
            continue;
        }
        if names.len() as u64 >= ceiling {
            return Err(resource("session entry enumeration ceiling exceeded"));
        }
        let stat = fs::statat(parent, name, AtFlags::SYMLINK_NOFOLLOW).map_err(io_error)?;
        names.push((name.to_owned(), FileType::from_raw_mode(stat.st_mode)));
    }
    names.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(names)
}
fn file_size(parent: &OwnedFd, name: &str, ceiling: u64) -> Result<u64> {
    let stat = fs::statat(parent, name, AtFlags::SYMLINK_NOFOLLOW).map_err(io_error)?;
    let size = u64::try_from(stat.st_size).map_err(|_| resource("negative file size"))?;
    if FileType::from_raw_mode(stat.st_mode) != FileType::RegularFile
        || stat.st_nlink != 1
        || size > ceiling
    {
        return Err(resource("nonregular, linked, or oversized session input"));
    }
    Ok(size)
}
fn read(parent: &OwnedFd, name: &str, ceiling: u64) -> Result<Vec<u8>> {
    let fd = fs::openat(
        parent,
        name,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(io_error)?;
    let mut file = File::from(fd);
    let meta = file.metadata().map_err(io_error)?;
    use std::os::unix::fs::MetadataExt;
    if !meta.is_file() || meta.nlink() != 1 || meta.len() > ceiling {
        return Err(resource("nonregular, linked, or oversized session input"));
    }
    let mut bytes = Vec::new();
    (&mut file)
        .take(ceiling + 1)
        .read_to_end(&mut bytes)
        .map_err(io_error)?;
    if bytes.len() as u64 > ceiling {
        return Err(resource("session input grew beyond ceiling"));
    }
    Ok(bytes)
}
fn write_new(parent: &OwnedFd, name: &str, bytes: &[u8]) -> Result<()> {
    let fd = fs::openat(
        parent,
        name,
        OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::from_raw_mode(0o600),
    )
    .map_err(io_error)?;
    let mut file = File::from(fd);
    file.write_all(bytes).map_err(io_error)?;
    file.sync_all().map_err(io_error)
}
fn clear_directory(parent: &OwnedFd, ceiling: u64) -> Result<()> {
    let entries = names(parent, ceiling)?;
    // Validate the complete bounded inventory before deleting anything.
    for (name, _) in &entries {
        file_size(parent, name, u64::MAX)?;
    }
    for (name, _) in entries {
        fs::unlinkat(parent, name.as_str(), AtFlags::empty()).map_err(io_error)?;
    }
    fs::fsync(parent).map_err(io_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);

    struct Scratch(PathBuf);
    impl Scratch {
        fn new() -> Self {
            let parent = std::env::current_dir().unwrap().join("tmp");
            std::fs::create_dir_all(&parent).unwrap();
            let path = parent.join(format!(
                "retention-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn session(&self) -> SessionStore {
            SessionStore::create(self.0.join("session"), limits()).unwrap()
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }
    fn limits() -> SessionLimits {
        SessionLimits::new(100, 10_000, 16, 10, 3, 32, 64).unwrap()
    }
    fn corpus(store: &SessionStore) -> Vec<Vec<u8>> {
        let mut inputs: Vec<_> = std::fs::read_dir(store.corpus_path())
            .unwrap()
            .map(|entry| std::fs::read(entry.unwrap().path()).unwrap())
            .collect();
        inputs.sort();
        inputs
    }

    #[test]
    fn admission_accounts_for_active_two_retained_generations_and_control_replacements() {
        let limits = SessionLimits::new(40, 1_000, 10, 100, 3, 10, 10).unwrap();
        // 11 active attempts + 3 active seeds + 4 engine allowances + 6 retained
        // replacement slots + 16 fixed entries = 40. Controls consume 62 bytes.
        assert_eq!(limits.round_cases(), 11);
        assert_eq!(limits.retained_entries(), 3);
        assert_eq!(limits.reserved_entries, 40);
        assert_eq!(limits.reserved_bytes, 302);
        assert!(SessionLimits::new(25, 1_000, 10, 100, 3, 10, 10).is_err());
        assert!(SessionLimits::new(100, 100, 10, 100, 3, 10, 10).is_err());
        assert!(SessionLimits::new(100, 10_000, 10, 2, 3, 10, 10).is_err());
    }

    #[test]
    fn seed_selection_deduplicates_and_reports_loss_in_small_first_order() {
        let scratch = Scratch::new();
        let mut store = scratch.session();
        let report = store
            .initialize(&[vec![8; 8], vec![3; 3], vec![1], vec![2; 2], vec![1]])
            .unwrap();
        assert_eq!(corpus(&store), vec![vec![1], vec![2; 2], vec![3; 3]]);
        assert_eq!((report.retained_entries, report.retained_bytes), (3, 6));
        assert_eq!((report.discarded_entries, report.discarded_bytes), (2, 9));
        let basis = b"recorded-target-build";
        store.write_basis(basis).unwrap();
        assert_eq!(
            std::fs::read(
                store
                    .corpus_path()
                    .parent()
                    .unwrap()
                    .join("corpus-basis.json")
            )
            .unwrap(),
            basis
        );
    }

    #[test]
    fn restart_larger_than_active_round_is_selected_before_materialization() {
        let scratch = Scratch::new();
        let mut store = scratch.session();
        let seeds: Vec<_> = (0..30_u8).map(|value| vec![value; 2]).collect();
        let report = store.initialize(&seeds).unwrap();
        assert_eq!(report.retained_entries, 3);
        assert_eq!(report.discarded_entries, 27);
        assert_eq!(store.report().unwrap().active_entries, 3);
    }

    #[test]
    fn repeated_rotation_remains_bounded_and_never_touches_evidence() {
        let scratch = Scratch::new();
        let evidence = scratch.0.join("immutable-failure");
        std::fs::write(&evidence, b"exact failing canonical bytes").unwrap();
        let mut store = scratch.session();
        store.initialize(&[vec![0; 8]]).unwrap();
        for round in 0..100_u8 {
            for candidate in 0..10_u8 {
                std::fs::write(
                    store.corpus_path().join(format!("candidate-{candidate}")),
                    vec![round, candidate],
                )
                .unwrap();
            }
            store.finish_round().unwrap();
            store
                .replace_checkpoint(format!("round:{round}").as_bytes())
                .unwrap();
            store.replace_log(b"bounded diagnostic tail").unwrap();
            store.reset_round().unwrap();
            let report = store.report().unwrap();
            assert!(report.retained_entries <= 3);
            assert!(report.retained_bytes <= 48);
            assert!(report.reserved_bytes <= 10_000);
            assert!(report.reserved_entries <= 100);
            assert!(report.partial_replacements.is_empty());
            assert_eq!(report.active_entries, report.retained_entries);
        }
        assert_eq!(
            std::fs::read(evidence).unwrap(),
            b"exact failing canonical bytes"
        );
    }

    #[test]
    fn terminal_retention_path_is_capped_and_engine_artifact_remains_exact() {
        let scratch = Scratch::new();
        let mut store = scratch.session();
        store.initialize(&[vec![0; 8]]).unwrap();
        for candidate in 1..8_u8 {
            std::fs::write(
                store.corpus_path().join(format!("candidate-{candidate}")),
                vec![candidate; candidate as usize],
            )
            .unwrap();
        }
        let engine_artifact = store.engine_artifacts_path().join("crash-entropy");
        std::fs::write(&engine_artifact, b"exact entropy").unwrap();
        store.finish_round().unwrap();
        let retained = store.retained_corpus_path().unwrap();
        let mut inputs: Vec<_> = std::fs::read_dir(&retained)
            .unwrap()
            .map(|entry| std::fs::read(entry.unwrap().path()).unwrap())
            .collect();
        inputs.sort();
        assert_eq!(inputs, vec![vec![1], vec![2; 2], vec![3; 3]]);
        assert_ne!(retained, store.corpus_path());
        assert_eq!(retained.parent(), store.corpus_path().parent());
        assert_eq!(std::fs::read(engine_artifact).unwrap(), b"exact entropy");
    }

    #[test]
    fn oversized_seed_and_control_write_preserve_committed_data() {
        let scratch = Scratch::new();
        let mut store = scratch.session();
        store.initialize(&[vec![1]]).unwrap();
        assert!(store.initialize(&[vec![7; 17]]).is_err());
        assert_eq!(corpus(&store), vec![vec![1]]);
        store.replace_checkpoint(b"committed").unwrap();
        assert!(store.replace_checkpoint(&[0; 65]).is_err());
        assert_eq!(
            std::fs::read(scratch.0.join("session/progress.json")).unwrap(),
            b"committed"
        );
        assert!(store.replace_log(&[0; 33]).is_err());
    }

    #[test]
    fn partial_checkpoint_is_reported_and_does_not_replace_committed_data() {
        let scratch = Scratch::new();
        let mut store = scratch.session();
        store.replace_checkpoint(b"committed").unwrap();
        std::fs::write(
            scratch.0.join("session/progress.json.partial"),
            b"interrupted",
        )
        .unwrap();
        assert_eq!(
            store.report().unwrap().partial_replacements,
            vec!["progress.json.partial"]
        );
        assert!(store.replace_checkpoint(b"next").is_err());
        assert_eq!(
            std::fs::read(scratch.0.join("session/progress.json")).unwrap(),
            b"committed"
        );
    }

    #[test]
    fn incomplete_rotation_does_not_claim_staged_inputs_as_committed() {
        let scratch = Scratch::new();
        let mut store = scratch.session();
        store.initialize(&[vec![1]]).unwrap();
        // Initialization committed b, so a is the uncommitted generation.
        std::fs::write(scratch.0.join("session/retained-a/interrupted"), [9]).unwrap();
        let report = store.report().unwrap();
        assert_eq!(report.retained_entries, 1);
        assert_eq!(report.partial_replacements, vec!["retained-a"]);
        store.reset_round().unwrap();
        assert_eq!(corpus(&store), vec![vec![1]]);
    }

    #[test]
    fn link_inputs_and_link_ancestors_are_rejected_without_touching_destination() {
        use std::os::unix::fs::symlink;
        let scratch = Scratch::new();
        let outside = scratch.0.join("outside");
        std::fs::write(&outside, b"preserve").unwrap();
        let mut store = scratch.session();
        store.initialize(&[vec![1]]).unwrap();
        let link = store.corpus_path().join("link");
        symlink(&outside, &link).unwrap();
        assert!(store.finish_round().is_err());
        assert!(store.reset_round().is_err());
        assert_eq!(std::fs::read(&outside).unwrap(), b"preserve");
        std::fs::remove_file(link).unwrap();
        std::fs::hard_link(&outside, store.corpus_path().join("hardlink")).unwrap();
        assert!(store.finish_round().is_err());
        symlink(&scratch.0, scratch.0.join("alias")).unwrap();
        assert!(SessionStore::create(scratch.0.join("alias/new"), limits()).is_err());
        assert!(!scratch.0.join("new").exists());
    }

    #[test]
    fn unexpected_and_excessive_entries_are_rejected_before_cleanup() {
        let scratch = Scratch::new();
        let mut store = scratch.session();
        store.initialize(&[vec![1]]).unwrap();
        for index in 0..18 {
            std::fs::write(store.corpus_path().join(format!("entry-{index}")), [1]).unwrap();
        }
        assert!(store.report().is_err());
        assert!(store.reset_round().is_err());
        assert!(store.corpus_path().join("entry-0").exists());
        assert!(SessionStore::create(scratch.0.join("session"), limits()).is_err());
    }
}
