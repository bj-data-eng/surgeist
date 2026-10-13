//! Controlled process behavior for supervisor tests only.
#![forbid(unsafe_code)]

use std::io::{Read, Write};
use std::process::Command;
use std::time::Duration;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("echo") => {
            let mut bytes = Vec::new();
            std::io::stdin().read_to_end(&mut bytes).unwrap();
            std::io::stdout().write_all(&bytes).unwrap();
            std::io::stderr().write_all(b"controlled stderr").unwrap();
        }
        Some("environment") => {
            print!(
                "{}:{}",
                std::env::var("SURGEIST_WORKER_VALUE").unwrap(),
                std::env::var("PATH").is_ok()
            );
        }
        Some("nonzero") => std::process::exit(17),
        Some("panic") => panic!("controlled worker panic"),
        Some("abort") => std::process::abort(),
        Some("ignore-term") => {
            ctrlc::set_handler(|| {}).unwrap();
            std::io::stdout().write_all(b"ready").unwrap();
            std::io::stdout().flush().unwrap();
            loop {
                std::thread::sleep(Duration::from_secs(60));
            }
        }
        Some("hang") => {
            if let Some(marker) = args.get(2) {
                std::fs::write(marker, b"ready").unwrap();
            }
            loop {
                std::thread::sleep(Duration::from_secs(60));
            }
        }
        Some("flood") => {
            let bytes = [b'x'; 8192];
            loop {
                if args.get(2).map(String::as_str) == Some("stderr") {
                    std::io::stderr().write_all(&bytes).unwrap();
                } else {
                    std::io::stdout().write_all(&bytes).unwrap();
                }
            }
        }
        Some("stream") => {
            let stdout_bytes: usize = args[2].parse().unwrap();
            let stderr_bytes: usize = args[3].parse().unwrap();
            let chunk = [b'x'; 8192];
            let mut stdout = std::io::stdout().lock();
            for offset in (0..stdout_bytes).step_by(chunk.len()) {
                stdout
                    .write_all(&chunk[..chunk.len().min(stdout_bytes - offset)])
                    .unwrap();
            }
            stdout.flush().unwrap();
            let chunk = [b'z'; 8192];
            let mut stderr = std::io::stderr().lock();
            for offset in (0..stderr_bytes).step_by(chunk.len()) {
                stderr
                    .write_all(&chunk[..chunk.len().min(stderr_bytes - offset)])
                    .unwrap();
            }
            stderr.write_all(b"last diagnostics").unwrap();
            stderr.flush().unwrap();
        }
        Some("framed-stream") => {
            let frames: usize = args[2].parse().unwrap();
            let response = surgeist_fuzz::worker::WorkerResponse::Generated {
                canonical: vec![3; 2048],
            };
            let mut stdout = std::io::stdout().lock();
            for _ in 0..frames {
                surgeist_fuzz::worker::write_frame(&mut stdout, &response, 8192).unwrap();
            }
        }
        Some("stream-drain") => {
            std::io::stdout().write_all(&[b'x'; 16 * 1024]).unwrap();
            std::io::stdout().flush().unwrap();
            std::fs::write(&args[2], b"output complete").unwrap();
            loop {
                std::thread::sleep(Duration::from_secs(60));
            }
        }
        Some("descendant") | Some("descendant-exit") => {
            println!("{}", abandoned_descendant(args.get(2)));
            std::io::stdout().flush().unwrap();
            if args[1] == "descendant-exit" {
                return;
            }
            loop {
                std::thread::sleep(Duration::from_secs(60));
            }
        }
        _ => std::process::exit(19),
    }
}

// Deliberately leaves a running descendant behind: this is the stimulus proving
// the executor owns group cleanup after an otherwise successful parent exit.
#[expect(
    clippy::zombie_processes,
    reason = "controlled orphan fixture; the executor verifies group disappearance"
)]
fn abandoned_descendant(marker: Option<&String>) -> u32 {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command.arg("hang");
    if let Some(marker) = marker {
        command.arg(marker);
    }
    let child = command.spawn().unwrap();
    child.id()
}
