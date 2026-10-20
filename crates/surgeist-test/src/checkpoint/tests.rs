use super::*;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    mpsc,
};
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
            "surgeist-checkpoint-{}-{now}-{next}",
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
                eprintln!("owned checkpoint-test directory cleanup failed: {error}");
            } else {
                panic!("owned checkpoint-test directory cleanup failed: {error}");
            }
        }
    }
}

fn config(directory: &Directory) -> Config {
    Config::new(
        directory.0.clone(),
        "unit-token".into(),
        Duration::from_secs(1),
    )
    .unwrap()
}

#[derive(Clone, Copy)]
struct ScalarIdentity {
    count: usize,
}

impl Identity for ScalarIdentity {
    fn record(self, token: &str, pid: u32, stage: &str, ordinal: Option<usize>) -> String {
        let mut result = format!(
            "token={token}\npid={pid}\nfixture=scalar\ncount={}\nstage={stage}\n",
            self.count
        );
        if let Some(ordinal) = ordinal {
            result.push_str(&format!("checkpoint={ordinal}\n"));
        }
        result
    }
}

fn identity() -> ScalarIdentity {
    ScalarIdentity { count: 7 }
}

const RECORD: &str = "token=unit-token\npid=42\nfixture=scalar\ncount=7\nstage=edited\n";

#[test]
fn configuration_validates_bounds_without_touching_the_directory() {
    let directory = Directory::new();
    let missing = directory.0.join("not-created");
    assert!(Config::new(missing.clone(), "A_0-z".into(), Duration::from_secs(1)).is_ok());
    assert!(Config::new(missing.clone(), "x".repeat(64), Duration::from_secs(300)).is_ok());
    assert!(!missing.exists());
    for (path, token, timeout) in [
        (PathBuf::new(), "token".into(), Duration::from_secs(1)),
        (missing.clone(), String::new(), Duration::from_secs(1)),
        (missing.clone(), "x".repeat(65), Duration::from_secs(1)),
        (missing.clone(), "bad\ntoken".into(), Duration::from_secs(1)),
        (missing.clone(), "é".into(), Duration::from_secs(1)),
        (missing.clone(), "token".into(), Duration::ZERO),
        (missing.clone(), "token".into(), Duration::from_millis(999)),
        (missing.clone(), "token".into(), Duration::from_secs(301)),
    ] {
        assert_eq!(
            Config::new(path, token, timeout).unwrap_err().kind(),
            io::ErrorKind::InvalidInput
        );
    }
    assert!(fs::read_dir(&directory.0).unwrap().next().is_none());
    assert_eq!(
        Gate::open(
            Config::new(missing, "token".into(), Duration::from_secs(1)).unwrap(),
            vec!["edited"]
        )
        .err()
        .unwrap()
        .kind(),
        io::ErrorKind::NotFound
    );
}

#[test]
fn file_gate_publishes_exact_record_and_keeps_runner_artifacts() {
    let directory = Directory::new();
    let pending = PendingAck::publish(
        &directory.0,
        identity().record("unit-token", 42, "edited", None),
        Duration::from_secs(1),
    )
    .unwrap();
    assert_eq!(
        fs::read(directory.0.join("ready")).unwrap(),
        RECORD.as_bytes()
    );
    assert!(!directory.0.join("ready.pending").exists());
    assert!(!pending.acknowledged().unwrap());
    fs::write(directory.0.join("ack.pending"), RECORD).unwrap();
    fs::rename(directory.0.join("ack.pending"), directory.0.join("ack")).unwrap();
    pending.wait().unwrap();
    assert!(directory.0.join("ready").is_file());
    assert!(directory.0.join("ack").is_file());
}

#[test]
fn acknowledgement_requires_every_byte_of_the_complete_record() {
    let directory = Directory::new();
    let expected =
        "token=unit-token\npid=42\nfixture=scalar\ncount=7\nstage=edited\ncheckpoint=1\n";
    let pending =
        PendingAck::publish(&directory.0, expected.into(), Duration::from_secs(1)).unwrap();
    for (old, new) in [
        ("unit-token", "foreign"),
        ("pid=42", "pid=43"),
        ("fixture=scalar", "fixture=other"),
        ("count=7", "count=8"),
        ("stage=edited", "stage=released"),
        ("checkpoint=1", "checkpoint=0"),
    ] {
        fs::write(directory.0.join("ack"), expected.replace(old, new)).unwrap();
        assert_eq!(
            pending.acknowledged().unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }
    for wrong in [expected.trim_end().to_owned(), format!("{expected}\n")] {
        fs::write(directory.0.join("ack"), wrong).unwrap();
        assert_eq!(
            pending.acknowledged().unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }
    fs::write(directory.0.join("ack"), expected).unwrap();
    assert!(pending.acknowledged().unwrap());
}

#[test]
fn file_gate_refuses_preexisting_oversized_and_nonregular_acknowledgements() {
    let directory = Directory::new();
    fs::write(directory.0.join("ack"), "stale").unwrap();
    assert_eq!(
        Gate::open(config(&directory), vec!["edited"])
            .err()
            .unwrap()
            .kind(),
        io::ErrorKind::AlreadyExists
    );

    let directory = Directory::new();
    let pending = PendingAck::publish(&directory.0, RECORD.into(), Duration::from_secs(1)).unwrap();
    fs::write(directory.0.join("ack"), vec![b'x'; 513]).unwrap();
    assert_eq!(
        pending.acknowledged().unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    fs::remove_file(directory.0.join("ack")).unwrap();
    fs::create_dir(directory.0.join("ack")).unwrap();
    assert_eq!(
        pending.acknowledged().unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );

    let directory = Directory::new();
    fs::write(directory.0.join("ack"), "premature").unwrap();
    assert_eq!(
        PendingAck::publish(&directory.0, RECORD.into(), Duration::from_secs(1))
            .err()
            .unwrap()
            .kind(),
        io::ErrorKind::AlreadyExists
    );
}

#[cfg(unix)]
#[test]
fn acknowledgement_rejects_a_symlink_even_to_matching_contents() {
    let directory = Directory::new();
    let pending = PendingAck::publish(&directory.0, RECORD.into(), Duration::from_secs(1)).unwrap();
    let target = directory.0.join("matching");
    fs::write(&target, RECORD).unwrap();
    std::os::unix::fs::symlink(target, directory.0.join("ack")).unwrap();
    assert_eq!(
        pending.acknowledged().unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
}

#[test]
fn ready_record_accepts_512_bytes_and_rejects_513_before_publication() {
    let directory = Directory::new();
    let expected = "x".repeat(512);
    let pending =
        PendingAck::publish(&directory.0, expected.clone(), Duration::from_secs(1)).unwrap();
    assert_eq!(fs::read(directory.0.join("ready")).unwrap().len(), 512);
    fs::write(directory.0.join("ack"), expected).unwrap();
    assert!(pending.acknowledged().unwrap());

    let directory = Directory::new();
    assert_eq!(
        PendingAck::publish(&directory.0, "x".repeat(513), Duration::from_secs(1))
            .err()
            .unwrap()
            .kind(),
        io::ErrorKind::InvalidInput
    );
    assert!(fs::read_dir(&directory.0).unwrap().next().is_none());
}

#[test]
fn missing_acknowledgement_at_expired_deadline_refuses_without_waiting() {
    let directory = Directory::new();
    let pending = PendingAck::publish(&directory.0, RECORD.into(), Duration::ZERO).unwrap();
    assert_eq!(pending.wait().unwrap_err().kind(), io::ErrorKind::TimedOut);
}

fn acknowledge_when_ready(directory: &Path, expected: &str) -> io::Result<()> {
    let started = Instant::now();
    loop {
        match fs::read(directory.join("ready")) {
            Ok(actual) => {
                if actual != expected.as_bytes() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "unexpected complete ready record",
                    ));
                }
                fs::write(directory.join("ack.pending"), expected)?;
                return fs::rename(directory.join("ack.pending"), directory.join("ack"));
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        if started.elapsed() >= Duration::from_secs(2) {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                format!(
                    "ready record missing in {} after {:?}",
                    directory.display(),
                    started.elapsed()
                ),
            ));
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn ordered_checkpoints_use_fresh_locations_and_reject_repeats_and_skips() {
    let directory = Directory::new();
    let mut gate = Gate::open(config(&directory), vec!["populated", "edited"]).unwrap();
    assert!(!gate.is_complete());
    assert_eq!(
        gate.checkpoint(identity(), 42, "edited")
            .unwrap_err()
            .kind(),
        io::ErrorKind::InvalidInput
    );
    gate.checkpoint(identity(), 42, "harness-only").unwrap();
    let first =
        "token=unit-token\npid=42\nfixture=scalar\ncount=7\nstage=populated\ncheckpoint=0\n";
    let second = "token=unit-token\npid=42\nfixture=scalar\ncount=7\nstage=edited\ncheckpoint=1\n";
    let root = directory.0.clone();
    let (sent, received) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        let result = acknowledge_when_ready(&root.join("00-populated"), first)
            .and_then(|()| acknowledge_when_ready(&root.join("01-edited"), second));
        let _ = sent.send(result);
    });
    let mut repeated = None;
    let result = gate.checkpoint(identity(), 42, "populated").and_then(|()| {
        repeated = Some(gate.checkpoint(identity(), 42, "populated"));
        gate.checkpoint(identity(), 42, "edited")
    });
    let worker_result = received.recv_timeout(Duration::from_secs(5));
    worker.join().unwrap();
    worker_result
        .expect("bounded runner completed both ready/ack operations")
        .unwrap();
    result.unwrap();
    assert_eq!(
        repeated.unwrap().unwrap_err().kind(),
        io::ErrorKind::InvalidInput
    );
    assert!(gate.is_complete());
    assert_eq!(
        fs::read(directory.0.join("00-populated/ready")).unwrap(),
        first.as_bytes()
    );
    assert_eq!(
        fs::read(directory.0.join("01-edited/ready")).unwrap(),
        second.as_bytes()
    );
    assert_eq!(
        gate.checkpoint(identity(), 42, "edited")
            .unwrap_err()
            .kind(),
        io::ErrorKind::InvalidInput
    );
}

#[test]
fn checkpoint_plan_refuses_invalid_names_duplicates_and_more_than_16_stages() {
    const STAGES: [&str; 17] = [
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q",
    ];
    for stages in [
        vec![],
        vec!["edited", "edited"],
        vec!["../edited"],
        vec!["EDITED"],
        STAGES.to_vec(),
    ] {
        let directory = Directory::new();
        assert_eq!(
            Gate::open(config(&directory), stages).err().unwrap().kind(),
            io::ErrorKind::InvalidInput
        );
        assert!(fs::read_dir(&directory.0).unwrap().next().is_none());
    }
    let directory = Directory::new();
    let gate = Gate::open(config(&directory), STAGES[..16].to_vec()).unwrap();
    assert!(!gate.is_complete());
    assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 16);
}
