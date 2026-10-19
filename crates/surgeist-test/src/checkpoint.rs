//! Bounded file acknowledgements for test-fixture checkpoints.
//!
//! The caller owns fixture selection, scalar identity records, the fresh stage
//! directory and any measurement tools. This transport holds no fixture value
//! or process authority. It leaves ready/acknowledgement files for the runner.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const MAX_ACK_BYTES: usize = 512;
const MAX_CHECKPOINTS: usize = 16;

/// Validated transport configuration; construction performs no filesystem I/O.
#[derive(Debug)]
pub struct Config {
    directory: PathBuf,
    token: String,
    timeout: Duration,
}

impl Config {
    /// Select a runner-owned directory, a nonempty token of at most 64 ASCII
    /// letters/digits/hyphens/underscores, and a timeout from 1 through 300 seconds.
    ///
    /// Invalid configuration returns [`io::ErrorKind::InvalidInput`]. The
    /// directory must already exist and be empty when [`Gate::open`] runs.
    pub fn new(directory: PathBuf, token: String, timeout: Duration) -> io::Result<Self> {
        if directory.as_os_str().is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "checkpoint directory must be nonempty",
            ));
        }
        if token.is_empty()
            || token.len() > 64
            || !token
                .bytes()
                .all(|value| value.is_ascii_alphanumeric() || value == b'-' || value == b'_')
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "checkpoint token requires 1..64 ASCII letters/digits/hyphens/underscores",
            ));
        }
        if !(Duration::from_secs(1)..=Duration::from_secs(300)).contains(&timeout) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "acknowledgement timeout must be 1..300 seconds",
            ));
        }
        Ok(Self {
            directory,
            token,
            timeout,
        })
    }
}

/// A consumer-owned scalar fixture identity.
///
/// Implement this on a closed `Copy` record containing counts and static labels,
/// not borrowed semantic values. The record must bind all supplied arguments and
/// the complete fixture selection in at most 512 UTF-8 bytes. `ordinal` is `None`
/// for a single-stage plan. Serialization belongs to the fixture, so existing
/// runner wire formats can remain exact. The gate never retains the identity.
pub trait Identity: Copy {
    /// Serialize the complete record that the runner must acknowledge verbatim.
    fn record(self, token: &str, pid: u32, stage: &str, ordinal: Option<usize>) -> String;
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

/// An ordered plan of at most 16 distinct checkpoint names.
///
/// A selected stage advances only after an exact acknowledgement. Stages absent
/// from the plan are inert. This type owns only transport configuration and
/// progress; it neither retains fixture identities nor removes runner files.
pub struct Gate {
    config: Config,
    checkpoints: Vec<&'static str>,
    completed: usize,
}

impl Gate {
    /// Validate the plan and require a fresh, empty existing directory. Names
    /// must be nonempty ASCII lowercase letters/hyphens. Multi-stage locations
    /// are created before this returns, allowing setup before a fixture baseline.
    ///
    /// Invalid plans return [`io::ErrorKind::InvalidInput`]; a populated directory
    /// returns [`io::ErrorKind::AlreadyExists`]. Filesystem errors pass through.
    pub fn open(config: Config, checkpoints: Vec<&'static str>) -> io::Result<Self> {
        if checkpoints.is_empty()
            || checkpoints.len() > MAX_CHECKPOINTS
            || checkpoints.iter().enumerate().any(|(index, name)| {
                name.is_empty()
                    || !name
                        .bytes()
                        .all(|value| value.is_ascii_lowercase() || value == b'-')
                    || checkpoints[..index].contains(name)
            })
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "checkpoint plan requires distinct bounded stage names",
            ));
        }
        empty_directory(&config.directory)?;
        if checkpoints.len() > 1 {
            for (ordinal, stage) in checkpoints.iter().enumerate() {
                fs::create_dir(config.directory.join(format!("{ordinal:02}-{stage}")))?;
            }
        }
        Ok(Self {
            config,
            checkpoints,
            completed: 0,
        })
    }

    /// Publish a selected ready record and wait for its exact acknowledgement.
    ///
    /// Repeated or skipped selected stages return [`io::ErrorKind::InvalidInput`].
    /// A mismatched, oversized or nonregular acknowledgement returns
    /// [`io::ErrorKind::InvalidData`]; an expired wait returns
    /// [`io::ErrorKind::TimedOut`]. Failures leave progress unchanged and retain
    /// runner artifacts. Unselected stages perform no filesystem I/O.
    pub fn checkpoint(
        &mut self,
        identity: impl Identity,
        pid: u32,
        stage: &'static str,
    ) -> io::Result<()> {
        let Some(ordinal) = self.checkpoints.iter().position(|name| *name == stage) else {
            return Ok(());
        };
        if ordinal != self.completed {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "checkpoint {ordinal} ({stage}) arrived with {} acknowledged",
                    self.completed
                ),
            ));
        }
        let multiple = self.checkpoints.len() > 1;
        let directory = if multiple {
            self.config.directory.join(format!("{ordinal:02}-{stage}"))
        } else {
            self.config.directory.clone()
        };
        let expected = identity.record(&self.config.token, pid, stage, multiple.then_some(ordinal));
        PendingAck::publish(&directory, expected, self.config.timeout)?.wait()?;
        self.completed += 1;
        Ok(())
    }

    /// Whether every selected checkpoint has been acknowledged in order.
    pub fn is_complete(&self) -> bool {
        self.completed == self.checkpoints.len()
    }
}

struct PendingAck {
    path: PathBuf,
    expected: String,
    timeout: Duration,
}

impl PendingAck {
    fn publish(directory: &Path, expected: String, timeout: Duration) -> io::Result<Self> {
        // Reject pre-acks at actual readiness, not only at gate construction.
        empty_directory(directory)?;
        if expected.len() > MAX_ACK_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "ready record too long",
            ));
        }
        let temporary = directory.join("ready.pending");
        let ready = directory.join("ready");
        {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)?;
            file.write_all(expected.as_bytes())?;
            file.flush()?;
        }
        fs::rename(temporary, ready)?;
        Ok(Self {
            path: directory.join("ack"),
            expected,
            timeout,
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
                "acknowledgement must match the complete identity record exactly",
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

#[cfg(test)]
mod tests;
