//! Length-framed owner-binary stdin/stdout protocol. Canonical capture precedes
//! evaluation, so the controller can retain a concrete case even after a crash.
#![forbid(unsafe_code)]

use crate::engine::{admit_bytes, replay};
use crate::{
    CaseOutcome, CaseTarget, ExecutionCounts, FuzzError, FuzzErrorKind, Generation,
    GenerationRejection, Result, TargetDescriptor,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::io::{Read, Write};

/// Strict JSON request in a four-byte big-endian length frame.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "mode", rename_all = "kebab-case", deny_unknown_fields)]
pub enum WorkerRequest {
    Descriptor,
    Generate {
        entropy: Vec<u8>,
    },
    Evaluate {
        canonical: Vec<u8>,
    },
    Reduce {
        canonical: Vec<u8>,
        max_candidates: u32,
    },
}

// Serde's internally tagged unit visitor ignores remaining fields, even with
// deny_unknown_fields. An empty struct variant preserves the strict contract.
#[derive(Deserialize)]
#[serde(tag = "mode", rename_all = "kebab-case", deny_unknown_fields)]
enum RawWorkerRequest {
    Descriptor {},
    Generate {
        entropy: Vec<u8>,
    },
    Evaluate {
        canonical: Vec<u8>,
    },
    Reduce {
        canonical: Vec<u8>,
        max_candidates: u32,
    },
}

impl<'de> Deserialize<'de> for WorkerRequest {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        Ok(match RawWorkerRequest::deserialize(deserializer)? {
            RawWorkerRequest::Descriptor {} => Self::Descriptor,
            RawWorkerRequest::Generate { entropy } => Self::Generate { entropy },
            RawWorkerRequest::Evaluate { canonical } => Self::Evaluate { canonical },
            RawWorkerRequest::Reduce {
                canonical,
                max_candidates,
            } => Self::Reduce {
                canonical,
                max_candidates,
            },
        })
    }
}

/// Generate/reduce never evaluate. Evaluate sends Captured then Evaluated.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "kebab-case", deny_unknown_fields)]
pub enum WorkerResponse {
    Descriptor {
        descriptor: TargetDescriptor,
    },
    Generated {
        canonical: Vec<u8>,
    },
    GenerationRejected {
        rejection: GenerationRejection,
        counts: ExecutionCounts,
    },
    Captured {
        canonical: Vec<u8>,
    },
    Evaluated {
        outcome: CaseOutcome,
        counts: ExecutionCounts,
    },
    Candidate {
        canonical: Vec<u8>,
    },
    Reduced {
        emitted: u32,
    },
}

/// Protocol allocations and output are bounded independently of owner resources.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorkerLimits {
    max_request_bytes: u32,
    max_response_bytes: u32,
    max_total_response_bytes: u64,
    max_candidates: u32,
}

impl WorkerLimits {
    pub fn new(
        max_request_bytes: u32,
        max_response_bytes: u32,
        max_total_response_bytes: u64,
        max_candidates: u32,
    ) -> Result<Self> {
        if max_request_bytes == 0
            || max_response_bytes == 0
            || max_total_response_bytes < u64::from(max_response_bytes) + 4
        {
            return Err(protocol(
                FuzzErrorKind::InvalidLimits,
                "positive frame limits and total capacity for one frame are required",
            ));
        }
        Ok(Self {
            max_request_bytes,
            max_response_bytes,
            max_total_response_bytes,
            max_candidates,
        })
    }
    #[must_use]
    pub const fn max_request_bytes(self) -> u32 {
        self.max_request_bytes
    }
    #[must_use]
    pub const fn max_response_bytes(self) -> u32 {
        self.max_response_bytes
    }
    #[must_use]
    pub const fn max_total_response_bytes(self) -> u64 {
        self.max_total_response_bytes
    }
    #[must_use]
    pub const fn max_candidates(self) -> u32 {
        self.max_candidates
    }
}

impl Default for WorkerLimits {
    fn default() -> Self {
        Self {
            max_request_bytes: 1_048_576,
            max_response_bytes: 1_048_576,
            max_total_response_bytes: 4_194_304,
            max_candidates: 1024,
        }
    }
}

fn protocol(kind: FuzzErrorKind, detail: impl Into<String>) -> FuzzError {
    FuzzError::new(kind, "owner worker protocol", detail)
}
fn io_error(detail: &'static str, source: std::io::Error) -> FuzzError {
    FuzzError::with_source(FuzzErrorKind::Io, "owner worker protocol", detail, source)
}

/// Reads exactly one frame. Clean EOF is None; truncated frames, unknown or
/// duplicate JSON fields, trailing data, and oversized headers are errors.
/// The header ceiling is checked before allocating the frame payload.
pub fn read_frame<R: Read, T: DeserializeOwned>(reader: &mut R, maximum: u32) -> Result<Option<T>> {
    let mut header = [0_u8; 4];
    loop {
        match reader.read(&mut header[..1]) {
            Ok(0) => return Ok(None),
            Ok(_) => break,
            Err(cause) if cause.kind() == std::io::ErrorKind::Interrupted => {}
            Err(cause) => return Err(io_error("read frame header", cause)),
        }
    }
    reader
        .read_exact(&mut header[1..])
        .map_err(|cause| io_error("truncated frame header", cause))?;
    let length = u32::from_be_bytes(header);
    if length == 0 || length > maximum {
        return Err(protocol(
            FuzzErrorKind::Resource,
            "frame length exceeds quota or is zero",
        ));
    }
    let length = usize::try_from(length).map_err(|_| {
        protocol(
            FuzzErrorKind::Resource,
            "frame length exceeds address space",
        )
    })?;
    let mut payload = Vec::new();
    payload
        .try_reserve_exact(length)
        .map_err(|_| protocol(FuzzErrorKind::Resource, "frame allocation failed"))?;
    payload.resize(length, 0);
    reader
        .read_exact(&mut payload)
        .map_err(|cause| io_error("truncated frame payload", cause))?;
    serde_json::from_slice(&payload).map(Some).map_err(|cause| {
        FuzzError::with_source(
            FuzzErrorKind::Codec,
            "owner worker protocol",
            "invalid JSON frame",
            cause,
        )
    })
}

struct BoundedBuffer {
    bytes: Vec<u8>,
    maximum: usize,
}
impl Write for BoundedBuffer {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.maximum.saturating_sub(self.bytes.len()) {
            return Err(std::io::Error::other("response frame quota exceeded"));
        }
        self.bytes
            .try_reserve_exact(bytes.len())
            .map_err(|_| std::io::Error::other("response allocation failed"))?;
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Serializes under a ceiling before writing a header or payload. Returns bytes
/// written including the header. It flushes before returning, including captures.
pub fn write_frame<W: Write, T: Serialize>(writer: &mut W, value: &T, maximum: u32) -> Result<u64> {
    let maximum = usize::try_from(maximum).map_err(|_| {
        protocol(
            FuzzErrorKind::Resource,
            "response ceiling exceeds address space",
        )
    })?;
    let mut payload = BoundedBuffer {
        bytes: Vec::new(),
        maximum,
    };
    serde_json::to_writer(&mut payload, value).map_err(|cause| {
        FuzzError::with_source(
            FuzzErrorKind::Resource,
            "owner worker protocol",
            "response exceeds allocation quota or cannot encode",
            cause,
        )
    })?;
    let length = u32::try_from(payload.bytes.len())
        .map_err(|_| protocol(FuzzErrorKind::Resource, "response length overflow"))?;
    writer
        .write_all(&length.to_be_bytes())
        .map_err(|cause| io_error("write frame header", cause))?;
    writer
        .write_all(&payload.bytes)
        .map_err(|cause| io_error("write frame payload", cause))?;
    writer
        .flush()
        .map_err(|cause| io_error("flush frame", cause))?;
    Ok(u64::from(length) + 4)
}

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "kebab-case")]
enum BorrowedResponse<'a> {
    Descriptor {
        descriptor: &'a TargetDescriptor,
    },
    Captured {
        canonical: &'a [u8],
    },
    #[cfg(feature = "coverage")]
    GenerationRejected {
        rejection: &'a GenerationRejection,
        counts: ExecutionCounts,
    },
    #[cfg(feature = "coverage")]
    Evaluated {
        outcome: &'a CaseOutcome,
        counts: ExecutionCounts,
    },
}

#[cfg(feature = "coverage")]
pub(crate) fn write_coverage_observation<W: Write>(
    writer: &mut W,
    result: &crate::engine::GeneratedEvaluation,
    counts: ExecutionCounts,
    maximum: u32,
) -> Result<u64> {
    use crate::engine::GeneratedEvaluation;
    match result {
        GeneratedEvaluation::Rejected(rejection) => write_frame(
            writer,
            &BorrowedResponse::GenerationRejected { rejection, counts },
            maximum,
        ),
        GeneratedEvaluation::Evaluated(evaluation) => write_frame(
            writer,
            &BorrowedResponse::Evaluated {
                outcome: &evaluation.outcome,
                counts,
            },
            maximum,
        ),
    }
}

/// Emits concrete capture from a borrowed slice under the frame quota.
pub fn write_capture<W: Write>(writer: &mut W, canonical: &[u8], maximum: u32) -> Result<u64> {
    write_frame(writer, &BorrowedResponse::Captured { canonical }, maximum)
}

struct ResponseWriter<'a, W> {
    writer: &'a mut W,
    limits: WorkerLimits,
    written: u64,
}
impl<W: Write> ResponseWriter<'_, W> {
    fn emit<T: Serialize>(&mut self, response: &T) -> Result<()> {
        let remaining = self
            .limits
            .max_total_response_bytes
            .checked_sub(self.written)
            .ok_or_else(|| protocol(FuzzErrorKind::Resource, "total response quota exceeded"))?;
        if remaining <= 4 {
            return Err(protocol(
                FuzzErrorKind::Resource,
                "total response quota exceeded",
            ));
        }
        let maximum = self
            .limits
            .max_response_bytes
            .min(u32::try_from(remaining - 4).unwrap_or(u32::MAX));
        self.written += write_frame(self.writer, response, maximum)?;
        Ok(())
    }
}

/// Owner binaries call this from main. Domain registration and semantic fixtures
/// remain in the owning binary; this service never spawns processes or writes files.
pub fn serve<T: CaseTarget>(target: &T) -> Result<()> {
    serve_io(
        target,
        &mut std::io::stdin().lock(),
        &mut std::io::stdout().lock(),
        WorkerLimits::default(),
    )
}

/// Handles one request per owner process. A second request is rejected rather
/// than allowing hidden state to bleed across cases. Request EOF is mandatory.
pub fn serve_io<T: CaseTarget, R: Read, W: Write>(
    target: &T,
    reader: &mut R,
    writer: &mut W,
    limits: WorkerLimits,
) -> Result<()> {
    let request = read_frame::<_, WorkerRequest>(reader, limits.max_request_bytes)?
        .ok_or_else(|| protocol(FuzzErrorKind::Codec, "missing request"))?;
    // One isolated process owns one request; no unbounded input stream.
    if read_frame::<_, WorkerRequest>(reader, limits.max_request_bytes)?.is_some() {
        return Err(protocol(
            FuzzErrorKind::Codec,
            "worker accepts exactly one request",
        ));
    }
    let mut output = ResponseWriter {
        writer,
        limits,
        written: 0,
    };
    match request {
        WorkerRequest::Descriptor => output.emit(&BorrowedResponse::Descriptor {
            descriptor: target.descriptor(),
        }),
        WorkerRequest::Generate { entropy } => {
            admit_bytes(target, &entropy)?;
            match target.generate(&entropy)? {
                Generation::Generated(case) => {
                    let canonical = target.encode(&case)?;
                    admit_bytes(target, &canonical)?;
                    output.emit(&WorkerResponse::Generated { canonical })
                }
                Generation::Rejected(rejection) => {
                    output.emit(&WorkerResponse::GenerationRejected {
                        rejection,
                        counts: ExecutionCounts::new(0, 1),
                    })
                }
            }
        }
        WorkerRequest::Evaluate { canonical } => {
            let evaluation = replay(target, &canonical, &mut |canonical| {
                output.emit(&BorrowedResponse::Captured { canonical })
            })?;
            let mut counts = ExecutionCounts::default();
            counts.record_outcome(&evaluation.outcome)?;
            output.emit(&WorkerResponse::Evaluated {
                outcome: evaluation.outcome,
                counts,
            })
        }
        WorkerRequest::Reduce {
            canonical,
            max_candidates,
        } => {
            if max_candidates > limits.max_candidates {
                return Err(protocol(
                    FuzzErrorKind::InvalidLimits,
                    "reduction candidate count exceeds quota",
                ));
            }
            admit_bytes(target, &canonical)?;
            let case = target.decode(&canonical)?;
            let encoded = target.encode(&case)?;
            if encoded != canonical {
                return Err(protocol(
                    FuzzErrorKind::InvalidCase,
                    "reduction input is not canonical",
                ));
            }
            let mut emitted = 0_u32;
            let mut failure = None;
            if max_candidates > 0 {
                target.reduction_candidates(&case, &mut |candidate| {
                    if emitted >= max_candidates || failure.is_some() {
                        return false;
                    }
                    let result = target.encode(&candidate).and_then(|canonical| {
                        admit_bytes(target, &canonical)?;
                        output.emit(&WorkerResponse::Candidate { canonical })
                    });
                    match result {
                        Ok(()) => {
                            emitted += 1;
                            emitted < max_candidates
                        }
                        Err(cause) => {
                            failure = Some(cause);
                            false
                        }
                    }
                })?;
            }
            if let Some(cause) = failure {
                return Err(cause);
            }
            output.emit(&WorkerResponse::Reduced { emitted })
        }
    }
}
