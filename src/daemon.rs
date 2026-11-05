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

fn running_pid(pid_path: &Path) -> Option<u32> {
    let pid: u32 = fs::read_to_string(pid_path).ok()?.trim().parse().ok()?;
    let alive = Command::new("kill")
        .args(["-0", &pid.to_string()])
        .stderr(Stdio::null())
        .status()
        .ok()?
        .success();
    alive.then_some(pid)
}
