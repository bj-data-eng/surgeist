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
            "surgeist-memory-gate-{}-{now}-{next}",
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

fn config(directory: &Directory) -> Config {
    Config::parse(
        true,
        Some(directory.0.to_str().unwrap()),
        Some("unit-token"),
        Some("1"),
    )
    .unwrap()
    .unwrap()
}

fn native() -> Identity {
    Identity::Native {
        population: "detached-text",
        managed: 1000,
        sample: Sample::First,
    }
}

fn parsed() -> Identity {
    Identity::Parsed {
        rows: 4,
        columns: 8,
        cells: 32,
        parsed_nodes: 75,
        managed: 76,
        sample: Sample::First,
    }
}

const NATIVE: &str =
    "token=unit-token\npid=42\npopulation=detached-text\nmanaged=1000\nsample=0\nstage=edited\n";
const PARSED: &str = "token=unit-token\npid=42\nfixture=parsed-table\nrows=4\ncolumns=8\ncells=32\nparsed_nodes=75\nmanaged=76\nsample=0\nstage=edited\ncheckpoint=0\n";
const LAYERED: &str = "token=unit-token\npid=42\nfixture=native-layered\nmode=combined\nids=row-ids\nrows=2\ncolumns=3\ncells=6\nmanaged=16\nsample=0\nstage=edited\ncheckpoint=1\n";

#[test]
fn file_gate_preserves_the_native_record_and_keeps_runner_artifacts() {
    let directory = Directory::new();
    let pending = PendingAck::publish(
        &directory.0,
        native().record("unit-token", 42, "edited", None),
        Duration::from_secs(1),
    )
    .unwrap();
    assert_eq!(
        fs::read(directory.0.join("ready")).unwrap(),
        NATIVE.as_bytes()
    );
    assert!(!pending.acknowledged().unwrap());
    fs::write(directory.0.join("ack.pending"), NATIVE).unwrap();
    fs::rename(directory.0.join("ack.pending"), directory.0.join("ack")).unwrap();
    pending.wait().unwrap();
    assert!(directory.0.join("ready").is_file());
    assert!(directory.0.join("ack").is_file());
}

#[test]
fn native_acknowledgement_refuses_foreign_selection_and_stage() {
    let directory = Directory::new();
    let pending = PendingAck::publish(&directory.0, NATIVE.into(), Duration::from_secs(1)).unwrap();
    for (old, new) in [
        ("unit-token", "foreign"),
        ("pid=42", "pid=43"),
        ("detached-text", "attached-text"),
        ("managed=1000", "managed=10000"),
        ("sample=0", "sample=1"),
        ("stage=edited", "stage=released"),
    ] {
        fs::write(directory.0.join("ack"), NATIVE.replace(old, new)).unwrap();
        assert_eq!(
            pending.acknowledged().unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }
}

#[test]
fn parsed_acknowledgement_binds_every_geometry_and_selection_field() {
    let directory = Directory::new();
    let pending = PendingAck::publish(
        &directory.0,
        parsed().record("unit-token", 42, "edited", None),
        Duration::from_secs(1),
    )
    .unwrap();
    assert_eq!(
        fs::read(directory.0.join("ready")).unwrap(),
        PARSED.as_bytes()
    );
    fs::write(directory.0.join("ack"), PARSED).unwrap();
    assert!(pending.acknowledged().unwrap());

    for (old, new) in [
        ("unit-token", "foreign"),
        ("pid=42", "pid=43"),
        ("fixture=parsed-table", "fixture=native"),
        ("rows=4", "rows=5"),
        ("columns=8", "columns=9"),
        ("cells=32", "cells=33"),
        ("parsed_nodes=75", "parsed_nodes=76"),
        ("managed=76", "managed=77"),
        ("sample=0", "sample=1"),
        ("stage=edited", "stage=released"),
        ("checkpoint=0", "checkpoint=1"),
    ] {
        fs::write(directory.0.join("ack"), PARSED.replace(old, new)).unwrap();
        assert_eq!(
            pending.acknowledged().unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }
}

#[test]
fn layered_acknowledgement_binds_mode_id_pattern_geometry_and_checkpoint() {
    let identity = Identity::Layered {
        mode: "combined",
        ids: "row-ids",
        rows: 2,
        columns: 3,
        cells: 6,
        managed: 16,
        sample: Sample::First,
    };
    let directory = Directory::new();
    let pending = PendingAck::publish(
        &directory.0,
        identity.record("unit-token", 42, "edited", Some(1)),
        Duration::from_secs(1),
    )
    .unwrap();
    assert_eq!(
        fs::read(directory.0.join("ready")).unwrap(),
        LAYERED.as_bytes()
    );
    fs::write(directory.0.join("ack"), LAYERED).unwrap();
    assert!(pending.acknowledged().unwrap());
    for (old, new) in [
        ("unit-token", "foreign"),
        ("pid=42", "pid=43"),
        ("fixture=native-layered", "fixture=parsed-table"),
        ("mode=combined", "mode=banks-only"),
        ("ids=row-ids", "ids=cell-ids"),
        ("rows=2", "rows=3"),
        ("columns=3", "columns=4"),
        ("cells=6", "cells=7"),
        ("managed=16", "managed=17"),
        ("sample=0", "sample=1"),
        ("stage=edited", "stage=released"),
        ("checkpoint=1", "checkpoint=0"),
    ] {
        fs::write(directory.0.join("ack"), LAYERED.replace(old, new)).unwrap();
        assert_eq!(
            pending.acknowledged().unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }
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
    let pending = PendingAck::publish(&directory.0, NATIVE.into(), Duration::from_secs(1)).unwrap();
    fs::write(directory.0.join("ack"), vec![b'x'; MAX_ACK_BYTES + 1]).unwrap();
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
        PendingAck::publish(&directory.0, NATIVE.into(), Duration::from_secs(1))
            .err()
            .unwrap()
            .kind(),
        io::ErrorKind::AlreadyExists
    );
}

#[test]
fn missing_acknowledgement_at_expired_deadline_refuses_without_waiting() {
    let directory = Directory::new();
    let pending = PendingAck::publish(&directory.0, NATIVE.into(), Duration::ZERO).unwrap();
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
        gate.checkpoint(parsed(), 42, "edited").unwrap_err().kind(),
        io::ErrorKind::InvalidInput
    );
    gate.checkpoint(parsed(), 42, "harness-only").unwrap();
    let first = "token=unit-token\npid=42\nfixture=parsed-table\nrows=4\ncolumns=8\ncells=32\nparsed_nodes=75\nmanaged=76\nsample=0\nstage=populated\ncheckpoint=0\n";
    let second = "token=unit-token\npid=42\nfixture=parsed-table\nrows=4\ncolumns=8\ncells=32\nparsed_nodes=75\nmanaged=76\nsample=0\nstage=edited\ncheckpoint=1\n";
    let root = directory.0.clone();
    let (sent, received) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        let result = acknowledge_when_ready(&root.join("00-populated"), first)
            .and_then(|()| acknowledge_when_ready(&root.join("01-edited"), second));
        // No blocking work follows this completion signal.
        let _ = sent.send(result);
    });
    let mut repeated = None;
    let result = gate.checkpoint(parsed(), 42, "populated").and_then(|()| {
        repeated = Some(gate.checkpoint(parsed(), 42, "populated"));
        gate.checkpoint(parsed(), 42, "edited")
    });
    let worker_result = received
        .recv_timeout(Duration::from_secs(5))
        .expect("bounded runner completed both ready/ack operations");
    worker.join().unwrap();
    worker_result.unwrap();
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
        gate.checkpoint(parsed(), 42, "edited").unwrap_err().kind(),
        io::ErrorKind::InvalidInput
    );
}

#[test]
fn checkpoint_plan_refuses_duplicate_or_invalid_stage_names_before_publication() {
    for stages in [
        vec![],
        vec!["edited", "edited"],
        vec!["../edited"],
        vec!["EDITED"],
    ] {
        let directory = Directory::new();
        assert_eq!(
            Gate::open(config(&directory), stages).err().unwrap().kind(),
            io::ErrorKind::InvalidInput
        );
        assert!(fs::read_dir(&directory.0).unwrap().next().is_none());
    }
}
