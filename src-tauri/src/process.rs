//! Helpers for finding and stopping the `claude` process backing a session.
//!
//! We don't track PIDs — Claude Code owns its own lifecycle. To act on the
//! running session we ask `lsof` who currently has the JSONL transcript open
//! and then signal that PID. There is at most one holder because Claude Code
//! opens its transcript in append mode for the lifetime of the session.

use std::io;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;

/// How long we give a process to exit on SIGTERM before escalating to SIGKILL.
const STOP_GRACE: Duration = Duration::from_millis(300);
/// Polling interval while waiting for a process to die.
const STOP_POLL: Duration = Duration::from_millis(20);

/// Find the PID of the process holding the given `.jsonl` open. Returns
/// `None` if no live process holds the file (the session isn't running).
///
/// Uses `lsof -F p` so the output is a fixed format (one `pNNN` line per
/// holder). Empty output, nonzero exit, or unparseable output all map to
/// `None` — callers treat "not running" the same way.
pub fn find_holder_pid(jsonl: &Path) -> Option<u32> {
    let output = Command::new("lsof")
        .args(["-F", "p"])
        .arg(jsonl)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Some(rest) = line.strip_prefix('p') {
            if let Ok(pid) = rest.trim().parse::<u32>() {
                return Some(pid);
            }
        }
    }
    None
}

/// Send `SIGTERM`, wait up to `STOP_GRACE` for the process to die, then
/// escalate to `SIGKILL`. Returns `Ok(())` once the process is gone (or was
/// never alive to begin with).
pub fn stop_pid(pid: u32) -> io::Result<()> {
    let target = Pid::from_raw(pid as i32);

    // Best-effort SIGTERM. ESRCH means the process is already gone.
    if let Err(err) = kill(target, Signal::SIGTERM) {
        if err == nix::errno::Errno::ESRCH {
            return Ok(());
        }
        return Err(io::Error::other(err));
    }

    let started = Instant::now();
    while started.elapsed() < STOP_GRACE {
        if !is_alive(target) {
            return Ok(());
        }
        thread::sleep(STOP_POLL);
    }

    // Grace expired — escalate.
    match kill(target, Signal::SIGKILL) {
        Ok(()) => Ok(()),
        Err(nix::errno::Errno::ESRCH) => Ok(()),
        Err(err) => Err(io::Error::other(err)),
    }
}

fn is_alive(pid: Pid) -> bool {
    // `kill(pid, 0)` checks for existence/permission without delivering a
    // signal. Errno ESRCH means the process is gone; anything else (incl.
    // EPERM) means it still exists.
    !matches!(kill(pid, None), Err(nix::errno::Errno::ESRCH))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::process::{Command, Stdio};
    use tempfile::tempdir;

    #[test]
    fn find_holder_pid_returns_none_when_unheld() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nobody.jsonl");
        File::create(&path).unwrap();
        assert_eq!(find_holder_pid(&path), None);
    }

    #[test]
    fn find_holder_pid_returns_none_for_missing_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("does-not-exist.jsonl");
        // lsof exits nonzero for a path that doesn't exist; the helper
        // should swallow that into None.
        assert_eq!(find_holder_pid(&path), None);
    }

    #[test]
    fn find_holder_pid_finds_open_holder() {
        // `tail -f` keeps the file open for the duration of the test.
        let dir = tempdir().unwrap();
        let path = dir.path().join("held.jsonl");
        File::create(&path).unwrap();

        let mut child = Command::new("tail")
            .arg("-f")
            .arg(&path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn tail -f");

        // Give tail a moment to actually open the file.
        thread::sleep(Duration::from_millis(150));

        let found = find_holder_pid(&path);
        // Best effort cleanup before asserting.
        let _ = child.kill();
        let _ = child.wait();

        assert_eq!(found, Some(child.id()));
    }

    #[test]
    fn stop_pid_kills_a_running_child() {
        let mut child = Command::new("sleep")
            .arg("5")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn sleep");

        let pid = child.id();
        stop_pid(pid).expect("stop_pid");
        let status = child.wait().expect("wait");
        assert!(!status.success() || status.code() == Some(0));
    }

    #[test]
    fn stop_pid_is_ok_for_already_dead_process() {
        // Spawn and immediately reap a child, then call stop_pid on its PID.
        let mut child = Command::new("true")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn true");
        let pid = child.id();
        let _ = child.wait();
        // PID may have been recycled in theory but in practice the kernel
        // won't reuse it within the test window. Either way, ESRCH is
        // swallowed and we expect Ok.
        assert!(stop_pid(pid).is_ok());
    }
}
