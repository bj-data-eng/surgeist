//! DOM-owned memory fixture identities, sample and environment selection.

use std::{io, time::Duration};
use surgeist_test::checkpoint::Config;

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
    Layered {
        mode: &'static str,
        ids: &'static str,
        values: &'static str,
        rows: usize,
        columns: usize,
        cells: usize,
        managed: usize,
        sample: Sample,
    },
}

impl surgeist_test::checkpoint::Identity for Identity {
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
            Self::Layered {
                mode,
                ids,
                values,
                rows,
                columns,
                cells,
                managed,
                sample,
            } => format!(
                "token={token}\npid={pid}\nfixture=native-layered\nmode={mode}\nids={ids}\nvalues={values}\nrows={rows}\ncolumns={columns}\ncells={cells}\nmanaged={managed}\nsample={}\nstage={stage}\ncheckpoint={}\n",
                sample.name(),
                ordinal.unwrap_or(0)
            ),
        }
    }
}

/// Environment policy belongs to the DOM fixture; transport validates only
/// the resulting directory, token and duration. Disabled holds reject inherited
/// transport settings without performing filesystem I/O.
pub(super) fn checkpoint_config(
    enabled: bool,
    directory: Option<&str>,
    token: Option<&str>,
    timeout: Option<&str>,
) -> io::Result<Option<Config>> {
    let invalid = |message| io::Error::new(io::ErrorKind::InvalidInput, message);
    if !enabled {
        if directory.is_some() || token.is_some() || timeout.is_some() {
            return Err(invalid(
                "hold=none forbids inherited acknowledgement configuration",
            ));
        }
        return Ok(None);
    }
    let directory = directory
        .filter(|value| !value.is_empty())
        .ok_or_else(|| invalid("missing SURGEIST_MEMORY_STAGE_DIR"))?;
    let token = token.ok_or_else(|| invalid("missing SURGEIST_MEMORY_TOKEN"))?;
    let seconds: u64 = timeout
        .ok_or_else(|| invalid("missing SURGEIST_MEMORY_ACK_TIMEOUT_SECONDS"))?
        .parse()
        .map_err(|_| invalid("invalid acknowledgement timeout"))?;
    Config::new(directory.into(), token.into(), Duration::from_secs(seconds)).map(Some)
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

#[cfg(test)]
mod tests;
