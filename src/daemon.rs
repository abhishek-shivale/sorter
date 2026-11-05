use std::fs::{self, OpenOptions};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use crate::config::Config;
use crate::error::SorterError;
use crate::{files, paths};

const POLL_INTERVAL: Duration = Duration::from_secs(60);

pub fn start() -> Result<(), SorterError> {
    let config = Config::load(&paths::config_path())?;

    let dir = paths::app_dir();
    fs::create_dir_all(&dir)?;

    let pid_path = dir.join("sorter.pid");
    if let Some(pid) = running_pid(&pid_path) {
        return Err(SorterError::AlreadyRunning(pid));
    }

    let log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("sorter.log"))?;

    let child = Command::new(std::env::current_exe()?)
        .arg("run")
        .stdin(Stdio::null())
        .stdout(log.try_clone()?)
        .stderr(log)
        .process_group(0)
        .spawn()?;

    fs::write(&pid_path, child.id().to_string())?;
    println!("sorter started (pid {}), watching {}", child.id(), config.url);
    Ok(())
}

pub fn run() -> Result<(), SorterError> {
    let config = Config::load(&paths::config_path())?;

    loop {
        if let Err(problem) = files::sort_dir(Path::new(&config.url), &config.ignore) {
            eprintln!("{problem}");
        }
        std::thread::sleep(POLL_INTERVAL);
    }
}

pub fn stop() -> Result<(), SorterError> {
    let pid_path = paths::app_dir().join("sorter.pid");

    let Some(pid) = running_pid(&pid_path) else {
        let _ = fs::remove_file(&pid_path);
        return Err(SorterError::NotRunning);
    };

    if !Command::new("kill").arg(pid.to_string()).status()?.success() {
        return Err(SorterError::StopFailed(pid));
    }

    for _ in 0..50 {
        if running_pid(&pid_path).is_none() {
            fs::remove_file(&pid_path)?;
            println!("sorter stopped (pid {pid})");
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    Err(SorterError::StopFailed(pid))
}

// Checks the command, not only the pid: the OS may have reused the pid, and stop kills it.
fn running_pid(pid_path: &Path) -> Option<u32> {
    let pid: u32 = fs::read_to_string(pid_path).ok()?.trim().parse().ok()?;
    let output = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "command="])
        .output()
        .ok()?;
    String::from_utf8_lossy(&output.stdout)
        .trim()
        .ends_with("sorter run")
        .then_some(pid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn running_pid_rejects_other_processes() {
        let pid_path = std::env::temp_dir().join(format!("sorter-pid-test-{}", std::process::id()));
        fs::write(&pid_path, std::process::id().to_string()).unwrap();

        assert_eq!(running_pid(&pid_path), None);

        fs::remove_file(&pid_path).unwrap();
    }
}
