//! Bounded runner acknowledgements for scalar fixture checkpoints. This module
//! owns no DOM, report, snapshot, diagnostic graph or measurement process.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const MAX_ACK_BYTES: usize = 512;
const MAX_CHECKPOINTS: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Sample {
    Warmup,
    First,
    Second,
    Third,
}

impl Sample {
    pub(super) fn parse(value: &str) -> Result<Self, &'static str> {
        match value {
            "warmup" => Ok(Self::Warmup),
            "0" => Ok(Self::First),
            "1" => Ok(Self::Second),
            "2" => Ok(Self::Third),
            _ => Err("invalid SURGEIST_MEMORY_SAMPLE"),
        }
    }

    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Warmup => "warmup",
            Self::First => "0",
            Self::Second => "1",
            Self::Third => "2",
        }
    }

    pub(super) fn ordinal(self) -> usize {
        match self {
            Self::Warmup | Self::First => 0,
            Self::Second => 1,
            Self::Third => 2,
        }
    }
}

/// Closed fixture identities, containing only scalar counts and static labels.
#[derive(Clone, Copy, Debug)]
pub(super) enum Identity {
    Native {
        population: &'static str,
        managed: usize,
        sample: Sample,
    },
    Parsed {
        rows: usize,
        columns: usize,
        cells: usize,
        parsed_nodes: usize,
        managed: usize,
        sample: Sample,
    },
}

impl Identity {
    fn record(self, token: &str, pid: u32, stage: &str, ordinal: Option<usize>) -> String {
        match self {
            Self::Native {
                population,
                managed,
                sample,
            } => {
                // Preserve the existing single-stage native runner protocol.
                let mut record = format!(
                    "token={token}\npid={pid}\npopulation={population}\nmanaged={managed}\nsample={}\nstage={stage}\n",
                    sample.name()
                );
                if let Some(ordinal) = ordinal {
                    record.push_str(&format!("checkpoint={ordinal}\n"));
                }
                record
            }
            Self::Parsed {
                rows,
                columns,
                cells,
                parsed_nodes,
                managed,
                sample,
            } => format!(
                "token={token}\npid={pid}\nfixture=parsed-table\nrows={rows}\ncolumns={columns}\ncells={cells}\nparsed_nodes={parsed_nodes}\nmanaged={managed}\nsample={}\nstage={stage}\ncheckpoint={}\n",
                sample.name(),
                ordinal.unwrap_or(0)
            ),
        }
    }
}

#[derive(Debug)]
pub(super) struct Config {
    directory: PathBuf,
    token: String,
    timeout: Duration,
}

impl Config {
    /// Selection/stage validation belongs to the fixture and precedes this IO.
    pub(super) fn parse(
        enabled: bool,
        directory: Option<&str>,
        token: Option<&str>,
        timeout: Option<&str>,
    ) -> Result<Option<Self>, &'static str> {
        if !enabled {
            if directory.is_some() || token.is_some() || timeout.is_some() {
                return Err("hold=none forbids inherited acknowledgement configuration");
            }
            return Ok(None);
        }
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
            directory: directory.into(),
            token: token.into(),
            timeout: Duration::from_secs(seconds),
        }))
    }
}

pub(super) fn required(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("explicit UTF-8 {name} is required"))
}

pub(super) fn optional(name: &str) -> Option<String> {
    match std::env::var(name) {
        Ok(value) => Some(value),
        Err(std::env::VarError::NotPresent) => None,
        Err(std::env::VarError::NotUnicode(_)) => panic!("{name} must be UTF-8"),
    }
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

pub(super) struct Gate {
    config: Config,
    checkpoints: Vec<&'static str>,
    completed: usize,
}

impl Gate {
    pub(super) fn open(config: Config, checkpoints: Vec<&'static str>) -> io::Result<Self> {
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
            // Allocate every fresh location before the fixture baseline. The
            // runner retains these files; neither side reuses a prior ack.
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

    pub(super) fn checkpoint(
        &mut self,
        identity: Identity,
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

    pub(super) fn is_complete(&self) -> bool {
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
        // Reject pre-acks again at actual readiness, not only at configuration.
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
                "acknowledgement must match the complete token/PID/selection/stage/checkpoint record exactly",
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
