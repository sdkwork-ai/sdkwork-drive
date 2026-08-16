//! Shared helpers for sdkwork-drive-contract integration tests.
//!
//! The helpers in this module work around a known Windows-specific race
//! condition in `std::process::Command::output` that surfaces as
//! `Os { code: 0, kind: Uncategorized, message: "Operation completed
//! successfully" }` when too many `node` child processes are spawned
//! in parallel. The bug is in the standard library itself: see
//! `library\std\src\sys\process\mod.rs` in Rust 1.92, where the
//! shared `output` helper unwraps the result of reading from a child
//! pipe. Under heavy parallel load on Windows the pipe read returns
//! the spurious zero OS error and the unwrap panics.
//!
//! `run_node_command_in` avoids `Command::output` entirely and instead
//! spawns the child, captures stdout/stderr manually, and waits with
//! the error-tolerant `Child::wait` API. The spawn is retried a
//! bounded number of times with a small backoff so a burst of parallel
//! tests does not synchronize on the same retry cadence.

use std::ffi::OsStr;
use std::io::{self, Read};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

const WINDOWS_RACE_RETRY_ATTEMPTS: u32 = 20;
const WINDOWS_RACE_RETRY_BACKOFF: Duration = Duration::from_millis(100);

/// Run a `Command` and return its captured output, working around the
/// Windows-specific pipe-read race in `std::process::Command::output`.
///
/// On non-Windows platforms this falls through to `Command::output`
/// because the standard library bug is Windows-specific.
pub fn run_command_with_retry(mut command: Command) -> io::Result<CommandOutput> {
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    if !cfg!(windows) {
        let output = command.output()?;
        return Ok(CommandOutput {
            status: output.status,
            stdout: output.stdout,
            stderr: output.stderr,
        });
    }

    let mut last_error: Option<io::Error> = None;
    for attempt in 0..WINDOWS_RACE_RETRY_ATTEMPTS {
        match run_windows_safe(&mut command) {
            Ok(output) => {
                eprintln!(
                    "[sdkwork-drive-contract-test] node command succeeded on attempt {}",
                    attempt + 1
                );
                return Ok(output);
            }
            Err(error) => {
                let raw = error.raw_os_error();
                eprintln!(
                    "[sdkwork-drive-contract-test] node command failed on attempt {}: raw_os_error={:?} kind={:?} message={}",
                    attempt + 1,
                    raw,
                    error.kind(),
                    error
                );
                if !is_windows_spawn_race(&error) {
                    eprintln!(
                        "[sdkwork-drive-contract-test] error is not a Windows spawn race, failing closed"
                    );
                    return Err(error);
                }
                last_error = Some(error);
                // Use a linear backoff so a burst of parallel tests
                // does not synchronize their retries.
                let delay = WINDOWS_RACE_RETRY_BACKOFF.saturating_mul(attempt + 1);
                sleep(delay);
            }
        }
    }
    Err(last_error.expect("retry loop should record at least one error"))
}

/// Build a `node` command rooted at `current_dir` and run it with the
/// Windows-aware retry helper.
pub fn run_node_command_in<I, S, P>(current_dir: P, args: I) -> io::Result<CommandOutput>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
    P: AsRef<Path>,
{
    let mut command = Command::new("node");
    command.current_dir(current_dir);
    command.args(args);
    run_command_with_retry(command)
}

/// Output of a child process, mirroring the fields consumed by tests.
pub struct CommandOutput {
    pub status: std::process::ExitStatus,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

fn run_windows_safe(command: &mut Command) -> io::Result<CommandOutput> {
    let mut child = command.spawn()?;
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    if let Some(mut pipe) = child.stdout.take() {
        pipe.read_to_end(&mut stdout)?;
    }
    if let Some(mut pipe) = child.stderr.take() {
        pipe.read_to_end(&mut stderr)?;
    }
    let status = child.wait()?;
    Ok(CommandOutput {
        status,
        stdout,
        stderr,
    })
}

fn is_windows_spawn_race(error: &io::Error) -> bool {
    // The Windows race surfaces with raw OS code 0 and a locale-localized
    // "Operation completed successfully" message. Any OS error with raw
    // code 0 is a synthetic Rust error, and the documented Windows
    // process-pipe race is the only known source in this crate, so the
    // raw code is the stable discriminator. Other legitimate failures
    // carry a non-zero raw code and fail closed.
    error.raw_os_error() == Some(0)
}
